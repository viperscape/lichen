some_block
    # this is some block

    if thing ["one"
             "two"
             "three"]

    has_weight weight < 250.0  # defines that weight is valid and below some value
    no_stars !items.stars  # defines that items does not have stars (IsNot)
    comp:all [has_weight
             no_stars]

    if comp next:now other_block  # immediately heads to other_block
    or ["how about something else?" next:await something_else]  # waits for manual advancement to something_else

    # if failure to advance, then we pickup back where we left off after the Await
    emit "still here?"

    emit ["here, have a number and boolean" 2.0 false]
    emit name  # reference an environment variable to return
    if some_attrib "G'day, you look weary, `name"  # use name variable as apart of formatted response

    next:select {"Go to store" store,  # store would be the actual node name
                "Get out of here" exit}  # both Keys are seperated by a comma

    if this.visited "hi again"


    @coins + my-env.size  # increment coins by variable in my-env block, basic math is built in to lichen
    @coins 5  # swaps the value in
    @coins (inc) 1 2 3  # a custom function that takes multiple arguments

    needs_coins coins < 1
    has_name name
    when {needs_coins @coins + 2,  # perform addition if needs_coins is true
         has_name @name "new-name"}  # state swap on name

    next:now some_block  # start over if user selects wrong entry!
;

def my-env
    size 5
;