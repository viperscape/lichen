root
    has_weight weight < 100.0
    
    if has_weight next:now town
    
    emit "You look overloaded, `name"
;

store
    if !this.visited "G'day, you look weary, `name"
    or "Welcome back my friend, `name"

    comp:all !items.Dragonscale-Great-Sword !this.visited
    if comp [
      "Let me tell you about the rare Dragonscale Great Sword"
      "Are you interested?"
      next:await info-dragonscale
    ]

    comp:all this.visited items.Valerium-Great-Sword
    if comp "You are quite the master, I see!"

    emit "See you later!"
    next:now town
;

info-dragonscale
    emit ["There is a long history of Dragonscale"
         "It all started.."]

    next:now store
;

town
    next:select {"Head to Store?" store,
                "Leave the town?" exit-town}

    emit "A dustball blows by"
    next:now town
;

exit-town
    emit "`name heads off into the sunset"
;