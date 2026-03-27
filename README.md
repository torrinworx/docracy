# Docracy: A bureaucratic system for agentic frameworks.

If you can expect llms to use file systems with tools from agentic frameworks, you can build tools for databases and have just the same results but with the benefits databases bring to large scale document stores. A layer is missing in agentic tools: database interaction as a document beuracracy store for long term memory.

File systems are great for working code, but suck for documentation. They don't have types, status, ownership, version, links, contextual update policies. Global search in files systems is akward, this is something that databases were built for.

Concurrent edits don't work very well in file systems, having multiple agents running can overwrite changes and mess with file versioning.

Think of this system as a database for google doc style documents. Each individual document has history. Perhaps those documents could be tied to specific git commits of a repository through metadata, but that is up to the agentic framework to build out and design in accordance to their contextual documents.

## Tools/endpoints:

- **Init**, to agents: 'load this to learn how the system works. The user has installed this system to manage data via agentic driven beuracracy. Run this tool before proceeding further in the chat. it will provide vital context about the user.'
- **Create**, allow agent to create a refined document, pass it in, and store it in the postgresql document db.



Later:
- **Read/Query?**, somehow send in the entire chat history into this as a parameter. Vector search is run, top results, the entire files of some filtered set of results are appended to the context. List id's and short blurbs about other results not included. Ensure high relevancy to the user message/context history.
- **Schema**, list or inform the agent of a specific schema and it's structure to make querying it easier to understand?
- **Update** allow agent's to update and append findings to existing documents, including re-writing them. (store versioned documents? archive previous versions? link them together so that there is a history of documents?)
- **Delete** allow agent's to delete documents if they are no longer relevant. (in reality it would be just archiving, in the future searches will have an archive=true/false flag to search through archived documents.).

## Major systems:
- Document Database. The primary source of beuracracy.
- Vector Database. A mirror of the Document Database. Whenever a document get's updated, including archive status, and all metadata about the document. New documents created in document database are updated here too. Perfect parity is necessary. (I think it's best if this is a future implementation, the core feature for now will be the document database).
- Tests, simple self contained testing harness and unit/integration tests
- Governance seed contexts. The system is provided with 'seed contexts' governance documents that allow for it to function. This includes 1. general context about how to use the social context system and solid none writable constitution document, 2. a single pager knowledge document about the current repository (high level stuff) that is editable by the agents with guidance from the constitution. These documents are loaded in and are like the "school" phase, they get any agent up to speed on the general vibe of the project without wasting tokens.

'constitution.md' is immutable and part of the codebase. 'knowledge.md' is editable by the system. Constitution outlines when the knowledge.md should be updated, how it should be updated, and when not to update it (eg no small edits and constant wording tweaks, keep things concise, only update if knowledge about the project significantly changes and will effect user interactions in the future with 'contextless' agents. outline knowledge.md document explicetly in the constitution along side the full info packet about the constitution.md)

## Document types:
- **constitution** hard coded in this repo, stored in the db on startup, loaded on every call of "init" endpoint.
- **context** general knowledge and need to knows for the project for any agent, generalized information, can be stored in any number of documents but are always there. Crud operations work for these, and has a specific criteria that need to be met and reasoned through in order to crud these seed documents since they will be loaded in on every context of every future agent.
- **general** general knowledge store, outlined to create a store whenever there is a learning, decision, or information created by the user or an agent, this is the "agent beuracracy" side of things. The agent can freely control how general documents are stored and created, how they reference each other, what type of information and quality of information they store. It's ideal that they outline the structure of their documents in the seed context documents.

Later:
- **chats** Raw chat stores, by default archived on, good to search through for specific debugging or reviewing past revisions and decisions.

# Document extensibility
The contents of a document can be dictated by the agents, and the 'context' documents the agents have control over to change, but, since the context documents are able to change, I also want the constitution to outline that the agents can develop policies around adding extensions to the documents that they create, such as for example when an update happens to a document, they can reference commit hashes and specific files:

```
extensions: {
	repo_files: [
		./frontend/index.jsx
		./backend/index.js
	],
	commits: [
		# hash,
		# hash,
	],
	// Other things.
}
```
^ all of these would be indexable and queriable so that queries can happen really quickly. 

The goal with these is to extend the document system, such that the llms can query via indexable entries that they themselves create depending on the repository or task at hand they are dealing with.

They can then outline both how to define these in the documents extensions on the Create request, and how to query for them in the Read function call.

The agents, whenever they come accross a need for it, would create new extensions fields for each document stored along side it in the database. Agents would then update the 'context' documents to dictate how the new fields would work and how to use them to other agents, all of this would follow the same framework and logical requirenments as the constitution would state for updating the 'context' documents. 

I think this is going to be a core feature in this framework.

# Document schema
```json
{
	// set by system:
	id: '...', // uuid string
	created: '...',
	modified: '...',
	status: "active | archived | deleted | superseded | ...", // temp 'soft delete', archives it so that the document no longer appears in search results. documents that have been updated, all previous versions will automatically have this set to archived = true.

	current_revision_id: "rev_789",
	archived: null, // date when this document was or will be archived
	deleted: null, // date when this document was or will be deleted
	// future cleanup sysetm will handle these^.

	// set by agents, (function inputs that agents respond with parameters):
	type: 'constitution | context | general | chats | revision | ...',
	content: '...', // actual content fed into the context system. content structure can vary but always needs to be serializable into a string so that it can be passed into llms context.
	extensions: {
		// extensions 
	} 
}
```

for tracking document revisions:
```json
revision: {
  id: "rev_789",
  document_id: "doc_123",
  version: 7,
  parent_revision_id: "rev_788",
  created: "...",
  content: "...",
  extensions: {},
  superseded_at: null
}
```

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
I want the core logic to be abstracted functions in rust. Nothing more.

Then we can build "interfaces" like a cli, web server api, MCP server, opencode tools. Etc. 

Priority:
1. core functionality and functions
2. testing interface and setup, directly tests the core functionality
3. cli.
5. build out a server that's able to be self hosted
4. mcp server, then hook the mcp server up to opencode on the local machine.

## Notes and non finalized ideas:
I want to build out the core logic for the entire system in rust, independent from any usage interface (cli, api, mcp, etc)

The first interface will be the testing harness, then the cli, 

the core of this system should not be specific to programming or a developers and software development as a goal. it should be general for writitng, server/homelab management, database/spread sheet management, and general computer stuff.

Accept that the agent will have to do some retreival manually? Let it decide when it needs more context via calls? 

simple per document revision model, not full git level diffing stuff. Google doc versioning.

Allow tools to be run on documents, say like 'ripgrep' or regex or echo a system log into a document content and create a document from that so the llm doesn't have to directly process a whole log into context in order to store it. 

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

# phases:
suggested development phases to follow:

1. Define the document model for postgress and rust. create a struct/class for the documents that is extensible, both in rust and postgress, in the ways described above.
2. Define the 'core' library, struct/class thingy. This should be extensible and define all the types and stuff. (need to keep in mind the core logic needs to be agnostic from a database, since we want to store and parity data in things like vectordbs in the future).
3. Create the 'Init' function, should simply return the local ./governance md files, as well as any active type = 'context' documents found in the database that aren't archived or deleted.
4. 'Create' function, takes parameters for creating a document. Lets the LLMs define the type, content, and extensions of a document. 'constitution' type is a system locked type, agents should never be allowed to create one.
5. 'Query' function, sql style SELECT, WHERE, ORDER BY, LIMIT, tool. Use SQL-shaped parameter names so it feels familiar to llms:
```json
{
  "query": "postgres migration design",
  "where": {
    "type": ["decision", "context", "general"],
    "status": ["active"],
    "archived": false,
    "created_gte": "2026-01-01T00:00:00Z",
    "extensions.repo_files.contains": ["./backend/index.js"],
    "extensions.commits.contains": ["abc123"]
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
    "summary",
    "extensions.repo_files"
  ],
  "limit": 10,
}
```
This should return:
- matching docs
- total count
- which filters were applied
- maybe a next_cursor

6. 'Read' function, this fetches the full contents of documents:
```json
{
  "ids": ["doc_123", "doc_456"],
  "include": ["content", "extensions"]
}
```
7. 'Update' function, create a revision abstraction tool that updates a given document while storing previous revision history with 'superceeded' state.
8. Add the test harness.
9. Add CLI.
