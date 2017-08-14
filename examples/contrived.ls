root
    @player.weight 50
    @player.name "Io"
    next:now town
;

store
    has_weight player.weight < 100
    
    if !has_weight "You look overloaded, `player.name" next:now town
    
    if !this.visited "G'day, you look weary, `player.name"
    or "Welcome back my friend, `player.name"

    comp:all !player.items.Dragonscale-Great-Sword !this.visited
    if comp [
      "Let me tell you about the rare Dragonscale Great Sword"
      "Are you interested?"
      next:await info-dragonscale
    ]

    comp:all this.visited player.items.Valerium-Great-Sword
    if comp "You are quite the master, I see!"

    emit "See you later!"
;

info-dragonscale
    emit ["There is a long history of Dragonscale"
         "It all started.."]
    @player.items (add) Dragonscale-Great-Sword
;

town
    next:select {"Head to Store?" store,
                "Leave the town?" exit-town}

    emit "A dustball blows by"
    next:restart
;

exit-town
    emit "`player.name heads off into the sunset"
    next:exit
;