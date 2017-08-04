def meta

;

def player
    drunk false

;

tavern-enter
    if !this.visited "Welcome to the tavern"
    or "The tavern door swings open"

    if player.drunk "Back so soon? (bar tender)"

    next:now tavern
;

tavern
    next:select {"Have a drink" bar,
                "Look around" tavern-look,
                "Leave" tavern-exit}
    
    if player.drunk ["Talk to cloaked figure?"
       next:await tavern-cloaked-figure]

    next:restart
;

tavern-look
    emit "You see a bartender, taking her time to make drinks. She seems to have a sour face on."
    if player.drunk "You see a cloaked figure sitting in the corner."

;

tavern-cloaked-figure
    emit "What do you want, fool"

;

tavern-exit
    emit "See ya, stranger (bar tender)"
    next:now town
;

town
    emit "You see a small tavern"
    next:select {"Sleep?" sleep,
                "Enter tavern?" tavern-enter}
;

sleep
    next:exit

;

root
    next:now tavern-enter
;

bar
    @player.drunk true
    emit "You drink a strong fermented spirit"
;