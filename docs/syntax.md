#### Syntax for Lichen Source

##### Blocks

Blocks are regions of code that designate logic/actions
Currently there are two types of blocks, a block prefixed with ```def``` is for defining variables and setting meta in key/value format, each on a new line.
All other [blocks](https://github.com/viperscape/lichen/blob/master/docs/syntax.ls#L) are considered as nodes and follows standard logic rules.
Defining a block starts with the name identifier and ends with a semicolon, each individually on its corresponding line.

A [def block](https://github.com/viperscape/lichen/blob/master/docs/syntax.ls#L41) offers local environment variables to define

##### Variables

Currently there is support for basic [variable](https://github.com/viperscape/lichen/blob/master/src/var.rs#L7) types
- Boolean
- Float (32 bit)
- String
- Symbol

Whole numbers are parsed in as floats. Strings are built from quoted strings in source, and any non-quoted text is considered a symbol.


##### Comments

[Comments](https://github.com/viperscape/lichen/blob/master/docs/syntax.ls#L2) are prefixed with a ```#``` and end at the new line

##### Indented Regions

Code within a block should be [indented](https://github.com/viperscape/lichen/blob/master/docs/syntax.ls#L4-L6) to a standard 4 spaces. Code within a multiline region should be aligned with neighboring entries. Extra spaces are completely optional, a minimum of 1 space should be present between entries.


##### New Lines/Multilines

Almost all commands are based on a single line entry, if something spans multiple lines then a [bracket pair](https://github.com/viperscape/lichen/blob/master/docs/syntax.ls#L10-L11) should be used. The starting bracket *must* be inline with the originating statement.


##### Formal Logic

###### Base Logic

[Logic](https://github.com/viperscape/lichen/blob/master/docs/syntax.ls#L8-L9) defines flow through the node. Current logic is as such:
- Is and IsNot valid/exists/boolean response
- Greater/Lesser-Than numeric comparison

The resulting logic types become local variables for use in flow-logic.


###### Composites

[Composites](https://github.com/viperscape/lichen/blob/master/docs/syntax.ls#L10) are logic results tied together, they must be specified as requiring All/Any/None tags

###### Flow Logic

[If](https://github.com/viperscape/lichen/blob/master/docs/syntax.ls#L13) statements are used to control entry points and behavior. The result of a valid if statement are entries returned to the originating caller in the form of Variables. The last entry in an result region can be a block/node direction that will define the next entry point to evalulate. Entries in (except the last) the region mimic Emit functionality. The final entry in this region mimics either Next or Await functionality.

[Or](https://github.com/viperscape/lichen/blob/master/docs/syntax.ls#L14) statements must always immediately follow an If statement, and is flow for a failing If statement.

##### Other/Non-Logic

External to if-statements and logic entirely, a block can also contain standard responses.  
[Emit](https://github.com/viperscape/lichen/blob/master/docs/syntax.ls#L19) returns variables back to the caller, and can be a multiline region.

##### Next

The [Next](https://github.com/viperscape/lichen/blob/master/docs/syntax.ls#L13-L17) statement defines an optionally pausable region which requires advancement. The statement must be tagged with a next type: now, await or select.


To pass multiple node entries to select on, use the [select tag](https://github.com/viperscape/lichen/blob/master/docs/syntax.ls#L23-L24). Note the use of braclets ```{}``` to create the key-value map. The end of each value-list must be terminated with a comma, it's a variable sized entry-- so no two entries must be similarly sized. The final entry in the map does not need a comma. The internal Map type can take any Var type, and automatically converts the Key to a String for internal use.

```
{"my-list" "one" "two" "three",  # note the comma, tells the parser to start next KV group
"other-list" "four"}
```


##### Formatting/Reference

Referenced variables can be returned to the caller, as well can be formatted into strings. The ` [backtick symbol](https://github.com/viperscape/lichen/blob/master/docs/syntax.ls#L21) is used to specify a referenced variable when formatting a string.


##### Mutate from Functions

There are a few builtins to mutate state. To affect data you must prefix the referenced variable with an [```@``` symbol](https://github.com/viperscape/lichen/blob/master/docs/syntax.ls#L29). Currently functions are only called on the top-level of the node, node within statement regions/multilines. It's also possible to implement your own custom function, to call it you simply surround the function-name within parenthesis. Note, all referenced variables will be pulled from any [```def``` blocks](https://github.com/viperscape/lichen/blob/master/docs/syntax.ls#L41) within the environment. In addition to some [builtins](https://github.com/viperscape/lichen/blob/master/src/var.rs#L88-L100), you can also build [basic objects](https://github.com/viperscape/lichen/blob/master/tests/unit.rs#L380-L397) with the ```new``` keyword.


When the node is reached, these side-affect functions will run immediately. See a [custom ```inc``` function](https://github.com/viperscape/lichen/blob/master/tests/state.rs#L38-L64), which we build on the rust side of things as a closure. Custom functions are given access to the entire def block environment, which is useful for finding the actual value of a symbol, see [this example](https://github.com/viperscape/lichen/blob/master/tests/state.rs#L51), which gets the underlying number. ```args``` in this case are any additional Vars you provide in the call, [for example](https://github.com/viperscape/lichen/blob/master/tests/state.rs#L40) we provide 3 numbers to iterate over.

##### When Mutate on Logic

[When example](https://github.com/viperscape/lichen/blob/master/docs/syntax.ls#L35) shows how to control flow of mutations based on logic results. When takes a Map object, where the key points to the logic tested, and the value is a mutation function.
