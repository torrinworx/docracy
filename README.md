# Docracy: A bureaucratic system for agentic frameworks.

> A Postgres-backed, versioned document store for agentic systems.

Docracy keeps long-lived agent context in a database instead of a filesystem: typed documents, status, immutable revisions, and deterministic query/search. It boots agents with governance docs and active context, and it makes concurrent edits safer with `expected_revision` instead of silent overwrite.

## What it does

Ships `Init/Create/Read/Query/Update`, durable Postgres storage, arbitrary JSON `content`/`extensions`, governance seeding, keyword search, pagination, and structured CLI errors, backed by core and integration tests.

## Opinionated by design

- Database-first, not filesystem-first.
- Current head plus immutable revision chain, not opaque rewrites.
- Governance lives in the repo and is reflected in the DB.
- Core semantics live in Rust; interfaces are transport layers.
- Store `extensions` now; govern extension querying later.
- Vector mirroring is future work, not v1.

## Why this exists

If you can expect llms to use file systems with tools from agentic frameworks, you can build tools for databases and have just the same results but with the benefits databases bring to large scale document stores. A layer is missing in agentic tools: database interaction as a document beuracracy store for long term memory.

File systems are great for working code, but suck for documentation. They don't have types, status, ownership, version, links, contextual update policies. Global search in file systems is akward, this is something that databases were built for.

Concurrent edits don't work very well in file systems, having multiple agents running can overwrite changes and mess with file versioning.

Think of this system as a database for google doc style documents. Each individual document has history. Perhaps those documents could be tied to specific git commits of a repository through metadata, but that is up to the agentic framework to build out and design in accordance to their contextual documents.

## Current state: shipped in v1

Docracy v1.0 is the Postgres-backed document store, revision model, test harness, and CLI. The current contract is:

- Init/Create/Read/Query/Update are implemented.
- Documents have `type`, `status`, timestamps, `content`, `extensions`, and a current revision head.
- Updates create immutable revisions and require `expected_revision` so stale writes fail.
- `Init` returns governance markdown files and active `context` documents.
- Postgres migrations, full-text content search, filtering, ordering, pagination, and tests are in place.

Future ideas and non-finalized notes are kept separate below so the current v1 behavior is easy to read.

## Tools/endpoints

### Shipped in v1

- **Init**, to agents: 'load this to learn how the system works.' In v1 this returns the local `./governance` markdown files, as well as any active `type = context` documents found in the database that aren't archived or deleted. It also ensures the stored constitution matches the repo copy.
- **Create**, allow agent to create a refined document, pass it in, and store it in the postgresql document db.
- **Read**, this fetches the full contents of current documents by id.
- **Query**, sql style `SELECT`, `WHERE`, `ORDER BY`, `LIMIT`, tool for current documents. Keyword search runs over document content. Extension-field search (`extensions.*`) is intentionally deferred/unsupported in v1.
- **Update**, allow agent's to update and append findings to existing documents, including re-writing them. This stores revision history and requires `expected_revision` so stale writes fail instead of overwriting.
- **CLI contract**, commands return JSON on stdout, commands that take payloads read JSON on stdin or from an input file, failures emit structured JSON on stderr and exit non-zero.

### Future ideas

- **Retrieval helper / smart context loading**, somehow send in the entire chat history into this as a parameter. Vector search is run, top results, the entire files of some filtered set of results are appended to the context. List id's and short blurbs about other results not included. Ensure high relevancy to the user message/context history.
- **Schema**, list or inform the agent of a specific schema and it's structure to make querying it easier to understand?
- **Delete**, allow agent's to delete documents if they are no longer relevant. In reality it would probably just be archiving as a first-class tool, while searches can choose whether archived/deleted docs are included.

## Major systems

### Shipped in v1

- **Document Database.** The primary source of beuracracy.
- **Postgres backend.** The first durable storage layer for the document database.
- **Tests**, simple self contained testing harness and unit/integration tests.
- **Governance seed contexts.** The system is provided with 'seed contexts' governance documents that allow for it to function. This includes 1. general context about how to use the system and a solid non writable constitution document, 2. project-specific context documents that are editable by the agents with guidance from the constitution. These documents are loaded in and are like the "school" phase, they get any agent up to speed on the general vibe of the project without wasting tokens.

`CONSTITUTION.md` is immutable and part of the codebase. The DB copy is expected to reflect it.

### Future ideas

- **Vector Database.** A mirror of the Document Database. Whenever a document get's updated, including archive status, and all metadata about the document. New documents created in document database are updated here too. Perfect parity is necessary. I think it's best if this stays a future implementation; the core feature for now is the document database.

## Document types

### Shipped in v1

- **constitution** hard coded in this repo, stored in the db on init, loaded on every call of `Init`.
- **context** general knowledge and need to knows for the project for any agent, generalized information, can be stored in any number of documents but are always there while active. Crud operations work for these, and they have specific importance because they are loaded in on every context of every future agent.
- **general** general knowledge store, outlined to create a store whenever there is a learning, decision, or information created by the user or an agent, this is the "agent beuracracy" side of things. The agent can freely control how general documents are stored and created, how they reference each other, what type of information and quality of information they store. It's ideal that they outline the structure of their documents in the seed context documents.

### Future ideas

- **chats** Raw chat stores, by default archived on, good to search through for specific debugging or reviewing past revisions and decisions.
- Additional specialized types like `webpage`, `file`, `decision`, `task`, etc.

# Document extensibility

### Shipped in v1

The contents of a document can be dictated by the agents, and the `context` documents the agents have control over can define conventions around that. For example, when an update happens to a document, agents may want to reference commit hashes and specific files:

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

The goal with these is to extend the document system, such that the llms can attach repository/task metadata that later phases may choose to index and query.

They can then outline how to define these in the document extensions on the Create request and how to surface them in Read responses.

The agents, whenever they come accross a need for it, would create new extension fields for each document stored along side it in the database. Agents would then update the `context` documents to dictate how the new fields would work and how to use them to other agents, all of this would follow the same framework and logical requirenments as the constitution would state for updating the `context` documents.

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
  type: 'constitution | context | general | ...',
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

Current `Query` payload:

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

Need process for convertin document class and extensible documents into revisions? how to do this while allowing document extensions to store props in revisions?

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

I want the core logic to be abstracted functions in rust. Nothing more.

The first interface is the testing harness, then the cli.

### Future interfaces

Then we can build other interfaces like a web server api, MCP server, opencode tools. Etc.

Priority:

1. core functionality and functions
2. testing interface and setup, directly tests the core functionality
3. cli
4. build out a server that's able to be self hosted
5. mcp server, then hook the mcp server up to opencode on the local machine

## Future ideas and non finalized notes

These are not shipped v1 features. They are design direction and open ideas.

I want to build out the core logic for the entire system in rust, independent from any usage interface (cli, api, mcp, etc)

the core of this system should not be specific to programming or developers and software development as a goal. it should be general for writitng, server/homelab management, database/spread sheet management, and general computer stuff.

Accept that the agent will have to do some retreival manually? Let it decide when it needs more context via calls?

simple per document revision model, not full git level diffing stuff. Google doc versioning.

Allow tools to be run on documents, say like `ripgrep` or regex or echo a system log into a document content and create a document from that so the llm doesn't have to directly process a whole log into context in order to store it.

key thing to keep in mind: we aren't trying to normalize arbitrary data itself, we are trying to organize it, and search through it using tools designed to search through arbitrary data. Keyword search, date matching, id matching, role matching, vector relevance search, etc. it's the agent's/llm's job to use that arbitrary data arbitrarily.

It would be smart to design this so that in the future there can be multiple clients connected to the same db, allow for documents to be assigned to a given client, so that each agent can modify a given document however they see fit.

I could eventually something where when a document is updated, an agent is called to validate the update before it's written to ensure it conforms to a 'validator' or something, similar to validation in other databases but for arbitrary data. not that it would be safe with prompt injecting, but it would be cool for verification of abstract ideas like "writing style". Things that we can't easily create regex for.

or watchers could be used for other things? idk

TODO/Later: linked documents, similar to how Obsidean works I guess? Why not have a structured way to link/cite other documents throughout a given document:

```json
links: [
  'document_1',
  'document_2',
]

// Then throughout the content, any references formatted as [0] just automatically mean that that document was cited in this fact/info.
```

This wouldn't be just within the extensions, it would be a baked in thing in all documents. Could be included in the content, but also

This would allow for cross document linking and searching and potentially something like a document tree context loading mechanism? or at least searching. might aid in searches stronger than pure vector search, at least with known concepts.

Maybe there can also be some validation here when the documents get's created/updated that the ids in the links array are present in the db.

The index order of the id is the index that the llms should use to cite specific facts in their content. so to reference document_1 here in this sentence, I would just do this. [0].

## v1 foundation that is now shipped

Suggested development phases to follow. This is effectively the order that produced v1:

1. Define the document model for postgress and rust. create a struct/class for the documents that is extensible, both in rust and postgress, in the ways described above.
2. Define the `core` library, struct/class thingy. This should be extensible and define all the types and stuff. need to keep in mind the core logic needs to be agnostic from a database, since we want to store and parity data in things like vectordbs in the future.
3. Create the `Init` function, should simply return the local `./governance` md files, as well as any active type = `context` documents found in the database that aren't archived or deleted.
4. `Create` function, takes parameters for creating a document. Lets the LLMs define the type, content, and extensions of a document. `constitution` type is a system locked type, agents should never be allowed to create one.
5. `Query` function, sql style `SELECT`, `WHERE`, `ORDER BY`, `LIMIT`, tool. Use SQL-shaped parameter names so it feels familiar to llms. Extension-field search (`extensions.*`) is intentionally deferred/unsupported in v1.
6. `Read` function, this fetches the full contents of documents.
7. `Update` function, create a revision abstraction tool that updates a given document while storing previous revision history.
8. Add the test harness.
9. Add CLI.
