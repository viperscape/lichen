#### Syntax for Lichen Source

##### Blocks

Blocks are regions of code that designate logic/actions
Currently there are two types of blocks, a block prefixed with ```def``` is for defining variables and setting meta in key/value format-- Note: this is not formally implemented. All other blocks are considered as nodes and follows standard logic rules.
Defining a block starts with the name identifier and ends with a semicolon, each individually on its corresponding line.

```
some_block

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

Almost all commands are based on a single line entry, if something spans multiple lines then a bracket pair should be used. The starting bracket *must* be inline with the originating statement.

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

Composites are logic results tied together, they must be specified as requiring All/Any/None tags

```
some_block
    has_weight weight > 2.0

    # requires either to be valid/true
    comp:any [has_weight
              !other_attrib]

    if comp next:now other_block
;
```

###### Flow Logic

If statements are used to control entry points and behavior. The result of a valid if statement are entries returned to the originating caller in the form of Variables. The last entry in an result region can be a block/node direction that will define the next entry point to evalulate. Entries in (except the last) the region mimic Emit functionality. The final entry in this region mimics either Next or Await functionality.

```
some_block
    if some_attrib ["seeya!" next:now other_block]
;
```

Or statements must always immediately follow an If statement, and is flow for a failing If statement.

```
some_block
    if some_attrib ["seeya!" next:now other_block]
    or ["how about something else?" next:await something_else]
;
```

##### Other/Non-Logic

External to if-statements and logic entirely, a block can also contain standard responses.  
Emit returns variables back to the caller, and can be a multiline region.

```
some_block
    emit "hi"
    emit ["here, have a number and boolean" 2.0 false]
    emit `name  # reference an environment variable to return
;
```

##### Next

The Next statement defines an optionally pausable region which requires advancement. The statement must be tagged with a next type: now, await or select.

```
some_block
    if some_attrib ["seeya!" next:now other_block]  # immediately heads to other_block
    or ["how about something else?" next:await something_else]  # waits for manual advancement to something_else

    # if failure to advance, then we pickup back where we left off after the Await
    emit "still here?"
;
```

To pass multiple node entries to select on use the select tag. Note, this cannot be nested and must exist on the top-level of the node (eg: not in an If).

```
some_block
    next:select ["Go to store" store  # store would be the actual node name
                "Get out of here" exit]
    next:now some_block  # start over if user selects wrong entry!
;
```    

##### Formatting/Reference

Referenced variables can be returned to the caller, as well can be formatted into strings. The ` backtick symbol is used to specify a referenced variable.

```
some_block
    if some_attrib "G'day, you look weary, `name"  # use name variable as apart of response
;
```

##### Internal State

Each node tracks its visited-status upon evaluation, and can be accessed with the ```this.visited``` variable.

```
some_block
    if this.visited "hi again"
;
```