#### Syntax for Lichen Source

##### Blocks

Blocks are regions of code that designate logic/actions
Currently there are two types of blocks, a block prefixed with ```def``` is for defining variables and setting meta in key/value format-- Note: this is not formally implemented. All other blocks are considered as nodes and follows standard logic rules.
Defining a block starts with the name identifier and ends with a semicolon, each individually on its corresponding line.

```
some_block
...
;
```

##### Comments

Comments are prefixed with a ```#``` and end at the new line

```
some_block
    # this is some block
;
```

##### Indented Regions

Code within a block should be indented to a standard 4 spaces. Code within a multiline region should be aligned with neighboring entries. Extra spaces are completely optional, a minimum of 1 space should be present between entries.

```
some_block
    if thing [
        "one"
        "two"
        "three"
    ]
;
```

##### New Lines/Multilines

Almost all commands are based on a single line entry, if something spans multiple lines then a bracket pair should be used. Currently only if statement results and composite groups should use multiline entries.

```
some_block
    has_weight weight > 2.0
    comp:all [has_weight
              !other_attrib]
;
```

##### Formal Logic

###### Base Logic

Logic defines flow through the node. Current logic is as such:
- Is and IsNot valid/exists/boolean response
- Greater/Lesser-Than numeric comparison
The resulting logic types become local variables for use in flow-logic.

```
some_block
    has_weight weight < 250.0  # defines that weight is valid and below some value
    no_stars !items.stars  # defines that items does not have stars (IsNot)
;
```

###### Composites

Composites are logic results tied together, they must be specified as requiring All/Any/None

```
some_block
    has_weight weight > 2.0

    # requires either to be valid/true
    comp:any [has_weight
              !other_attrib]

    if comp next other_block
;
```

###### Flow Logic

If statements are used to control entry points and behavior. The result of a valid if statement are entries returned to the originating caller in the form of Variables. The last entry in an result region can be a block/node direction that will define the next entry point to evalulate. Entries in (except the last) the region mimic Emit functionality. The final entry in this region mimics either Next or Await functionality.

```
some_block
    if some_attrib ["seeya!" next other_block]
;
```

Or statements must always immediately follow an If statement, and is flow for a failing If statement.

```
some_block
    if some_attrib ["seeya!" next other_block]
    or ["how about something else?" await something_else]
;
```

##### Other/Non-Logic

External to if-statements and logic entirely, a block can also contain standard responses.

```
some_block
    emit "hi"
    next other_block
;
```

##### Await

The Await statement defines a pausable region which requires advancement, unlike a Next statement which is immediately directed to the next block.

```
some_block
    if some_attrib ["seeya!" next other_block]  # immediately heads to other_block
    or ["how about something else?" await something_else]  # waits for manual advancement to something_else

    # if failure to advance, then we pickup back where we left off after the Await
    emit "still here?"
;
```
