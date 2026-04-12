# Docracy: A document beuracracy for agents.

Docracy provides agentic tooling to record context into a database rather than a file system. This provides tools to scale context management, such as multi client and agent knoweldgbases, constant knowledge maintenance, and easier more relevant context retreival.

Using Docracy in combination with existing agentic frameworks, file system interaciton, and other tools, will help record the knowledge you generate throughout the chats you have with your agents.

The system is designed to become smarter the more you build with it, simply by recording the tokens you pay for.

# Agent Loops
These are loops facilitated by plugins curated to specific agentic tools such as Claude Code, OpenAI Codex, and OpenCode. Each plugin tries to accomplish the functionality of the following loops, to varying dagrees given the limitations of integrating docracy with these systems. Why plugins/addons to these agentic frameworks? Well, telling the agent to "Always load this docracy_init() function at the start of every chat before responding to the user" can work, but the downside is that it requires manual setup wherever you operate the agentic framework. Setting this system up purely as an mcp server doesn't easily or enforcably enable the loop that is required to maintain the document beuracracy this system aims to build for your use cases.

# Init
1. At the start of the chat, the Docracy Init core function is called to provide a base context to the agent main loop. This is always ran.

The context generated from this is injected once at the begining of the chat session, and is maintained at the top of the system context floating message throughout the progression of the chat. 

## Main user interaction loop:
This is the general context retreival loop. An agent can further investigate the context gathered in this step using the Docracy read tool to read the document directly.

1. User sends message.
2. System vector searches for relevant context of the past 10 messages in the chat. References and summaries of the documents within the results are injected into a floating 'system context' message.
3. User message + system context message get sent to agent.

The above steps get looped on each message the user sends in the chat.

4. Agent responds.
5. Next user message is sent, previous system context message is removed and replaced with a fresh query.

The philosophy behind this approach is: Vector querying is cheap. Do it often. Keep it cited.

## Record keeping/updating loop:
A general record keeping loop. This runs on completion of an agent response.

1. User sends a message.
2. Agent builds something, researches something, etc.
3. Agent finalizes a response to the user.

4. Sub agent is spun up, provided a system context message including relevant documents found in db semantically similar to the parent agents full task log inlcuding their response to the user.
5. Sub agent is prompted: Are there any findings, discoveries, decisions that were made in this session that relate to existing context in the db? Yes? No?
6. No? Create a new document if the information in the chat was useful, and summarize the key details using token efficient language/structure.
7. Yes? Please append those findings to the documents found in the database. If details in the documents need to be updated to reflect new findings, schemas, builds, implementations, do so carefully.

This loop is designed to create, maintain and delete/archive higher level synthesized documents. In the future there will be a full chat log record that will be referenced in each child document created from the above loop. 


# Roadmap:
- [ ] Via agentic framework plugins, allow for full chat storage and referencing to other higher level docs. These shouldn't appear in normal searches, and should be archived on creation so that they don't effect 'living memory'.
- [ ] OpenCode plugin, main and record keeping loops.
- [ ] Ability to manually edit/create documents.'
