# parse.rs

parse_str:
- accepts a context, an input string, and an optional name; returns a transition system
- tries to create a parser and use the input, returning the result if possible; and reporting errors if not

parse_file:
- accepts a path, returns a tuple of Context and TransitionSystem
- creates a context. attempts to parse a file with the context, and return both the context and the resulting transition system

parse_file_with_ctx:
- accepts a path and a context, returns a TransitionSystem
- attempts to open and read the file; and then parse it in a new parser.
- return the result if successful; otherwise report errors with the filename and path as additional parts.

## Parser
fields:
- ctx: a mutable reference to a Context
- sys: a TransitionSystem
- errors: a bunch of Errors
- offset: offset of current line in file
- type_map: maps line ID to a expr::Type
- state_map: maps line ID to a expr::StateRef in the transition system
- signal_map: maps line ID to an expr::ExprRef in the transition system
- unique_names: used in order to create unique names

LineID: alias for u32



