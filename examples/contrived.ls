root
    has_weight weight < 100.0
    
    if has_weight next store
    
    emit "You look overloaded, `name"
;

store
    no_dgs !items.Dragonscale-Great-Sword
    comp:all [no_dgs '!this.visited]
    if '!this.visited "G'day, you look weary, `name"
    if 'this.visited "Welcome back my friend, `name"

    if comp [
      "Let me tell you about the rare Dragonscale Great Sword"
      "Are you interested?"
      next info-dragonscale
    ]

    await

    if 'items.Valerium-Great-Sword "You are quite the master, I see!"
;

info-dragonscale
    emit ["There is a long history of Dragonscale"
         "It all started.."]

    next store
;