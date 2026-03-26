# Docracy: A bureaucratic system for agentic frameworks.

If you can expect llms to use file systems with tools from agentic frameworks, you can build tools for databases and have just the same results but with the benefits databases bring to large scale document stores. A layer is missing in agentic tools: database interaction as a document beuracracy store for long term memory.

File systems are great for working code, but suck for documentation. They don't have types, status, ownership, version, links, contextual update policies. Global search in files systems is akward, this is something that databases were built for.

Concurrent edits don't work very well in file systems, having multiple agents running can overwrite changes and mess with file versioning.

Think of this system as a database for google doc style documents. Each individual document has history. Perhaps those documents could be tied to specific git commits of a repository through metadata, but that is up to the agentic framework to build out and design in accordance to their contextual documents.

## Tools/endpoints:

- **init**, to agents: 'load this to learn how the system works. The user has installed this system to manage data via agentic driven beuracracy. Run this tool before proceeding further in the chat. it will provide vital context about the user.'
- **Create**, allow agent to create a refined document, pass it in, and store it in a normal db + vector database.
- **Read**, somehow send in the entire chat history into this as a parameter. Vector search is run, top results, the entire files of some filtered set of results are appended to the context. List id's and short blurbs about other results not included. Ensure high relevancy to the user message/context history.

Later:
- **Update** allow agent's to update and append findings to existing documents, including re-writing them. (store versioned documents? archive previous versions? link them together so that there is a history of documents?)
- **Delete** allow agent's to delete documents if they are no longer relevant. (in reality it would be just archiving, in the future searches will have an archive=true/false flag to search through archived documents.).

## Major systems:
- Document Database. The primary source of beuracracy. 
- Vector Database. A mirror of the Document Database. Whenever a document get's updated, including archive status, and all metadata about the document. New documents created in document database are updated here too. Perfect parity is necessary.
- Tests, simple self contained testing harness and unit/integration tests
- Seed contexts. The system is provided with 'seed contexts' governance documents that allow for it to function. This includes 1. general context about how to use the social context system and solid none writable constitution document, 2. a single pager knowledge document about the current repository (high level stuff) that is editable by the agents with guidance from the constitution. These documents are loaded in and are like the "school" phase, they get any agent up to speed on the general vibe of the project without wasting tokens. 

'constitution.md' is immutable and part of the codebase. 'knowledge.md' is editable by the system. Constitution outlines when the knowledge.md should be updated, how it should be updated, and when not to update it (eg no small edits and constant wording tweaks, keep things concise, only update if knowledge about the project significantly changes and will effect user interactions in the future with 'contextless' agents. outline knowledge.md document explicetly in the constitution along side the full info packet about the constitution.md).

## Document types:
- **constitution** hard coded in this repo, stored in the db on startup, loaded on every call of "init" endpoint.
- **context** general knowledge and need to knows for the project for any agent, generalized information, can be stored in any number of documents but are always there. Crud operations work for these, and has a specific criteria that need to be met and reasoned through in order to crud these seed documents since they will be loaded in on every context of every future agent.
- **general** general knowledge store, outlined to create a store whenever there is a learning, decision, or information created by the user or an agent, this is the "agent beuracracy" side of things. The agent can freely control how general documents are stored and created, how they reference each other, what type of information and quality of information they store. It's ideal that they outline the structure of their documents in the seed context documents.

Later:
- **chats** Raw chat stores, by default archived on, good to search through for specific debugging or reviewing past revisions and decisions.

# Document extensibility
The contents of a document can be dictated by the agents, and the 'context' documents the agents have control over to change, but, since the context documents are able to change, I also want the constitution to outline that the agents can develop policies around adding metadata to the documents that they create, such as for example when an update happens to a document, they can reference commit hashes and specific files:

```
metadata: {
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

They can then outline both how to define these in the documents metadata on the Create request, and how to query for them in the Read function call.

The agents, whenever they come accross a need for it, would create new metadata fields for each document stored along side it in the database. Agents would then update the 'context' documents to dictate how the new fields would work and how to use them to other agents, all of this would follow the same framework and logical requirenments as the constitution would state for updating the 'context' documents. 

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
	content: '...', // actual content fed into the context system.
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

## Future
It would be smart to design this so that in the future there can be multiple clients connected to the same db, allow for documents to be assigned to a given client, so that each agent can modify a given document however they see fit.

I could eventually something where when a document is updated, an agent is called to validate the update before it's written to ensure it conforms to a 'validator' or something, similar to validation in other databases but for arbitrary data. not that it would be safe with prompt injecting, but it would be cool for verification of abstract ideas like "writing style". Things that we can't easily create regex for.

or watchers could be used for other things? idk

## Notes:
I want to build out the core logic for the entire system in rust, independent from any usage interface (cli, api, mcp, etc)

The first interface will be the testing harness, then the cli, 

I feel like the above system still presents the problem of "The llm doesn't have the appropriate context to even contribute to the system", I feel like they would still act like a child in terms of not understanding the system or the repo itself. There isn't a vector that allows them to contribute to a growing standard or corpus. it's still independent and search base. Has no way for a communtiy to grow, constant memory is needed here for this to imerge somehow. Maybe we need some "init_context" intro stuff that intros the agent into the system and tells them how to interact with it?

How should documents be organized in the db? How would it make sense for an llm to reference it? From my experience llms in agentic frameworks really like things in a given folder directory structure. It's easy for them to navigate becase there are already lots of tools avaialbe in linux to navigate to specific files and find things and re-read things if needed. Putting things in a db kinda get's rid of that. Solution to this? 

the core of this system should not be specific to programming or a developers and software development as a goal. it should be general for writitng, server/homelab management, database/spread sheet management, and general computer stuff.

Accept that the agent will have to do some retreival manually? Let it decide when it needs more context via calls? 

Do we need version management? Why not just have everything in git? Maybe, core is sql or some database, documents are stored there. Then we have a "propegation layer" that mirrors those changes in something like git so we get version control? and at the same time in a vector database so that we can have vector search? That feels like an overbearing idea though. I have doubts, but it could be useful? 

I'm leaning towards this being an independnt opinionated system separate from git. I feel like we can do fancier things here without the need for mirroring functionality and having to keep parity with git work flows. Git would give us version control yes, but the part of the goal here, and what makes for good systems that llms like to use is simlifyijng and abstracing the complexities here. If an llm can use git to write all this, sure it would have complete control over everything, but it would also allow it to fuck up a merge, fuck up a rebase or a nasty force push. Yeah we aren't storing the diffs, and that means less efficiency, but these are mostly really small text files, While I like the idea of having it git compatible and version controlled I don't really see massive benefits here to having git asa the core base of everything document wise.

Allow tools to be run on documents, say like 'ripgrep' or regex or echo a system log into a document content and create a document from that so the llm doesn't have to directly process a whole log into context in order to store it. 

key thing to keep in mind: we aren't trying to normalize arbitrary data, we are trying to organize it, and search through it using tools designed to search through arbitrary data. Keyword search, date matching, id matching, role matching, vector relevance search, etc.
