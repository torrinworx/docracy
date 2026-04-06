# Docracy: A bureaucratic system for agentic frameworks.

Docracy lets your agents create, use, and store agent context artifacts in a database instead of a filesystem. It boots your agents with project-relevant context so that they can record their workflows and carry that context into the next task.

You can give an LLM tools to use a filesystem, so why not give it tools for working with databases?

## Current state:

Docracy v1.0 is the Postgres-backed document store, revision model, test harness, and CLI. The current contract is:

- Init/Create/Read/Query/Update are implemented.
- Documents have `type`, `status`, timestamps, `content`, `extensions`, and a current revision head.
- Updates create immutable revisions and require `expected_revision` so stale writes fail.
- `Init` returns the repo-owned `./governance` markdown files and active `context` documents created by other agents.
- Postgres migrations, full-text content search, filtering, ordering, pagination, and tests are in place.

Future ideas and non-finalized notes are kept separate below so the current v1 behavior is easy to read.

## Tools/endpoints

### Shipped in v1

- **Init**, to agents: 'load this to learn how the system works.' In v1 this returns the repo-owned `./governance` markdown files, as well as any active `type = context` documents found in the database that are not archived or deleted. It also ensures the stored governance document matches the repo copy.
- **Create**, allows agents to create a refined document, pass it in, and store it in the PostgreSQL document DB.
- **Read**, this fetches the full contents of current documents by id.
- **Query**, an SQL-style `SELECT`, `WHERE`, `ORDER BY`, `LIMIT` tool for current documents. Keyword search runs over document content. Extension-field search (`extensions.*`) is intentionally deferred/unsupported in v1.
- **Update**, allows agents to update and append findings to existing documents, including rewriting them. This stores revision history and requires `expected_revision` so stale writes fail instead of overwriting.
- **CLI contract**, commands return JSON on stdout, commands that take payloads read JSON on stdin or from an input file, failures emit structured JSON on stderr and exit non-zero.

### Future ideas

- **Retrieval helper / smart context loading**, somehow send the entire chat history into this as a parameter. Vector search runs, top results, and the full files of some filtered set of results are appended to the context. List IDs and short blurbs about other results not included. Ensure high relevance to the user message/context history.
- **Schema**, list or inform the agent of a specific schema and its structure to make querying it easier to understand?
- **Delete**, allow agents to delete documents if they are no longer relevant. In reality it would probably just be archiving as a first-class tool, while searches can choose whether archived/deleted docs are included.

## Major systems

### Shipped in v1

- **Document Database.** The primary source of bureaucracy.
- **Postgres backend.** The first durable storage layer for the document database.
- **Tests**, a simple self-contained testing harness and unit/integration tests.
- **Governance seed contexts.** The system is provided with repo-owned governance documents that allow it to function. This includes 1. general context about how to use the system and a solid non-writable governance document, 2. project-specific context documents that are editable by agents with guidance from the governance document. These documents are loaded in and are like the "school" phase; they get any agent up to speed on the general vibe of the project without wasting tokens.

The repo-owned governance bundle under `./governance` is immutable and part of the codebase. The DB copy is expected to reflect it.

### Future ideas

- **Vector Database.** A mirror of the Document Database. Whenever a document gets updated, including archive status and all metadata about the document, the mirror is updated too. Perfect parity is necessary. I think it's best if this stays a future implementation; the core feature for now is the document database.

## Document types

### Shipped in v1

- **governance** hard-coded in this repo, stored in the DB on init, loaded on every call of `Init`.
- **context** general knowledge and things an agent needs to know for the project; generalized information can be stored in any number of documents, but these are always present while active. CRUD operations work for these, and they are important because they are loaded into every context for every future agent.
- **general** general knowledge store, intended to create a store whenever there is a learning, decision, or piece of information created by the user or an agent. This is the "agent bureaucracy" side of things. The agent can freely control how general documents are stored and created, how they reference each other, and what type and quality of information they store. It is ideal that they outline the structure of their documents in the seed context documents.

### Future ideas

- **chats** Raw chat stores, archived by default, good to search through for specific debugging or reviewing past revisions and decisions.
- Additional specialized types like `webpage`, `file`, `decision`, `task`, etc.

# Document extensibility

### Shipped in v1

The contents of a document can be dictated by agents, and the `context` documents they control can define conventions around that. For example, when an update happens to a document, agents may want to reference commit hashes and specific files:

```json
extensions: {
  repo_files: [
    ./frontend/index.jsx,
    ./backend/index.js,
  ],
  commits: [
    # hash,
    # hash,
  ],
  // Other things.
}
```

In v1 these are stored and returned, but extension-field search/filtering is deferred until governance defines a policy.

### Future ideas

The goal with these is to extend the document system so that LLMs can attach repository/task metadata that later phases may choose to index and query.

They can then outline how to define these in the document extensions on the Create request and how to surface them in Read responses.

Whenever agents come across a need for it, they would create new extension fields for each document stored alongside it in the database. Agents would then update the `context` documents to dictate how the new fields would work and how to use them for other agents. All of this would follow the same framework and logical requirements as the governance document would state for updating the `context` documents.

I think this is going to be a core feature in this framework.

# Current v1 schema

Current document shape:

```json
{
  // set by system:
  id: '...', // uuid string
  created_at: '...',
  modified_at: '...',
  status: "active | archived | deleted | ...",

  current_revision_id: '...',
  archived_at: null,
  deleted_at: null,

  // set by agents, function inputs, or the calling interface:
  type: 'governance | context | general | ...',
  content: '...', // any non-null JSON
  extensions: {
    // extensions
  }
}
```

Current revision shape:

```json
revision: {
  id: '...',
  document_id: '...',
  version: 7,
  parent_revision_id: '...',
  created_at: '...',
  content: '...',
  extensions: {},
  superseded_at: null
}
```

In v1 the public interfaces are centered on current document create/read/query/update, while revisions are stored immutably underneath.

Current `Query` payload (raw SQL takes precedence when `sql` is present):

```json
{
  "sql": "SELECT id, \"type\", status FROM documents WHERE status = 'active' ORDER BY modified_at DESC",
  "limit": 25,
  "timeout_ms": 2500
}
```

Raw SQL executes in a read-only transaction, and the server clamps requests to 100 rows and 5000ms before execution.

If `sql` is omitted, the guided path still works:

```json
{
  "query": "postgres migration design",
  "where": {
    "type": ["decision", "context", "general"],
    "status": ["active"],
    "archived": false,
    "created_gte": "2026-01-01T00:00:00Z"
  },
  "order_by": [
    { "field": "modified", "direction": "desc" }
  ],
  "select": [
    "id",
    "type",
    "status",
    "created",
    "modified",
    "title",
    "summary"
  ],
  "limit": 10
}
```

This should return:

- matching docs
- total count
- which filters were applied
- maybe a `next_cursor`

Current `Read` payload:

```json
{
  "ids": [
    "123e4567-e89b-12d3-a456-426614174000",
    "123e4567-e89b-12d3-a456-426614174001"
  ]
}
```

The CLI returns the stored current document payloads; there is no `include` field in v1.

Current `Update` payload:

```json
{
  "id": "123e4567-e89b-12d3-a456-426614174000",
  "expected_revision": "123e4567-e89b-12d3-a456-426614174010",
  "content": {"title": "Updated doc"},
  "extensions": {"repo_files": ["./backend/index.js"]},
  "status": "active"
}
```

If the expected revision is stale, the CLI returns structured JSON like `{"error":{"kind":"revision_conflict",...}}` and exits non-zero.

## Future document type ideas

Need a process for converting document classes and extensible documents into revisions? How do we do this while allowing document extensions to store props in revisions?

classes and extending a base class, I would like there to be multiple types of documents, not just text based ones:

webpage:

```json
{
  ...document.class.base,
  type: 'webpage',
  content: {
    // structured html? content results from a scraper? some random structure...
  }
  url: 'https://...',
}
```

file:

```json
{
  ...document.class.base,
  type: 'file'
  ???
}
```

decision:

```json
{
  ...document.class.base,
  type: 'decision',
  content: {
    decision: 'we will be pursuing abc design',
    why: 'because concerns were brought up over xyz',
    references: [
      {
        id: '12345', // document id?
        relation: 'Outlines core issues with approach efg, making abc more appealing for ...',
      },
      ...
    ]
  }
}
```

task:

```json
{
  ...document.class.base,
  type: 'task',
}
```

log: system log, just one big blurb.

email?

chat: complete message history of a chat/agent workflow?

idk I can't think of any more?

## Interface

### Shipped in v1

I want the core logic to be abstracted functions in Rust. Nothing more.

The first interface is the testing harness, then the CLI.

### Future interfaces

Then we can build other interfaces like a web server API, MCP server, and OpenCode tools.

Priority:

1. Core functionality and functions
2. Testing interface and setup, directly tests the core functionality
3. CLI
4. Build out a server that's able to be self-hosted
5. MCP server, then hook the MCP server up to OpenCode on the local machine

## Future ideas and non finalized notes

These are not shipped v1 features. They are design direction and open ideas.

I want to build out the core logic for the entire system in Rust, independent from any usage interface (CLI, API, MCP, etc.).

The core of this system should not be specific to programming, developers, or software development as a goal. It should be general for writing, server/homelab management, database/spreadsheet management, and general computer tasks.

Accept that the agent will have to do some retrieval manually? Let it decide when it needs more context via calls?

Simple per-document revision model, not full git-level diffing stuff. Google Docs versioning.

Allow tools to be run on documents, such as `ripgrep` or regex, or echo a system log into document content and create a document from that so the LLM does not have to directly process a whole log into context in order to store it.

Key thing to keep in mind: we are not trying to normalize arbitrary data itself; we are trying to organize it and search through it using tools designed to search through arbitrary data. Keyword search, date matching, ID matching, role matching, vector relevance search, etc. It is the agent's/LLM's job to use that arbitrary data arbitrarily.

It would be smart to design this so that in the future there can be multiple clients connected to the same DB, allowing documents to be assigned to a given client so that each agent can modify a given document however they see fit.

I could eventually build something where, when a document is updated, an agent is called to validate the update before it's written to ensure it conforms to a "validator" or something, similar to validation in other databases but for arbitrary data. Not that it would be safe against prompt injection, but it would be cool for verification of abstract ideas like "writing style". Things that we cannot easily express with regex.

or watchers could be used for other things? idk

TODO/Later: linked documents, similar to how Obsidian works I guess? Why not have a structured way to link/cite other documents throughout a given document:

```json
links: [
  'document_1',
  'document_2',
]

// Then throughout the content, any references formatted as [0] just automatically mean that that document was cited in this fact/info.
```

This wouldn't be just within the extensions, it would be a baked in thing in all documents. Could be included in the content, but also

This would allow for cross document linking and searching and potentially something like a document tree context loading mechanism? or at least searching. might aid in searches stronger than pure vector search, at least with known concepts.

Maybe there can also be some validation here when the documents get created or updated that the IDs in the links array are present in the DB.

The index order of the ID is the index that the LLMs should use to cite specific facts in their content. So to reference document_1 here in this sentence, I would just do this: [0].

## v1 foundation that is now shipped

Suggested development phases to follow. This is effectively the order that produced v1:

1. Define the document model for Postgres and Rust. Create a struct/class for the documents that is extensible, both in Rust and Postgres, in the ways described above.
2. Define the `core` library, struct/class thingy. This should be extensible and define all the types and stuff. Need to keep in mind the core logic needs to be agnostic from a database, since we want to store parity data in things like vector DBs in the future.
   3. Create the `Init` function, which should simply return the repo-owned `./governance` MD files, as well as any active `type = context` documents found in the database that are not archived or deleted.
   4. `Create` function, takes parameters for creating a document. Let the LLMs define the type, content, and extensions of a document. `governance` type is a system-locked type, and agents should never be allowed to create one.
5. `Query` function, SQL-style `SELECT`, `WHERE`, `ORDER BY`, `LIMIT` tool. Use SQL-shaped parameter names so it feels familiar to LLMs. Extension-field search (`extensions.*`) is intentionally deferred/unsupported in v1.
6. `Read` function, this fetches the full contents of documents.
7. `Update` function, create a revision abstraction tool that updates a given document while storing previous revision history.
8. Add the test harness.
9. Add CLI.



# How to use:

Prerequisites: install and configure OpenCode.

For OpenCode:
1. Set up Docracy MCP config:
```json
"mcp": {
  "docracy": {
    "type": "local",
    "command": [
      "cargo",
      "run",
      "-p",
      "docracy-mcp",
      "--bin",
      "docracy-mcp",
      "--",
    ],
    "environment": {
      "DATABASE_URL": "postgresql://docracy:docracy_dev_password@localhost:5432/docracy"
    }
  }
}
```
2. Run the Docker Compose file:
```bash
cd ./docracy
docker compose up -d
```
3. Add an AGENTS.md file to your repo and place something like this in it:
```md
<!-- DOCRACY -->
Before responding to the user or conducting a task, run the docracy_init tool call. This will provide you with the necessary context managed by the Docracy system to operate effectively in this repository.  
```
4. Start chatting. Now the Docracy governance documents and your agent-generated context documents will be auto-loaded into all your future conversations with a given repo.
