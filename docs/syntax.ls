root
    now some_block
;

some_block
    # this is some block
    @some_block.visited true
    
    if player.name ["one"
                   "two"
                   "three"]

    has_weight player.weight < 250.0  # defines that weight is valid and below some value
    comp:all [!player.stars  # defines that items does not have stars (IsNot)
             has_weight]

    if comp now other_block  # immediately heads to other_block
    or ["how about something else?" await another_block]  # waits for manual advancement to something_else

    # if failure to advance, then we pickup back where we left off after the Await
    emit "still here?"

    emit ["here, have a number and boolean" 2.0 false]
    emit player.name  # reference an environment variable to return
    if player.name "G'day, you look weary, `player.name"  # use name variable as apart of formatted response

    select {"Go to store" store,  # store would be the actual node name
                "Get out of here" exit}  # both Keys are seperated by a comma

    if some_block.visited "hi again"


    @player.coins + my-env.size  # increment coins by variable in my-env block, basic math is built in to lichen
    @player.coins 5  # swaps the value in
    @player.coins (inc) 1 2 3  # a custom function that takes multiple arguments

    needs_coins player.coins < 1
    when {needs_coins @coins + 2,  # perform addition if needs_coins is true
         player.name @name "new-name"}  # state swap on name

    restart  # start over if user selects wrong entry!
;

def my-env
    size 5
;

def player
    name "Io"
    weight 50
;

other_block
;

another_block
;