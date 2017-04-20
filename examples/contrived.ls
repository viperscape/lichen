root
    has_weight weight < 100.0
    
    if has_weight next store
    
    emit "You look overloaded, `name"
;

store
    emit "G'day, you look weary, `name"

    if '!items.Dragonscale-Great-Sword [
      "Let me tell you about the rare Dragonscale Great Sword"
      "Are you interested?"
    ]

    await

    if 'items.Valerium-Great-Sword "You are quite the master, I see!"
;

info-dragonscale
    emit ["There is a long history of Dragonscale"
         "It all started.."]
;