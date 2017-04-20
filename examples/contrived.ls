root
    has_weight weight < 100.0
    
    if has_weight next store
    
    emit "You look overloaded, `name"
;

store
    emit "G'day, you look weary, `name"

    if '!items.Dragonscale-Great-Sword [
       "I have a great sword to sell,
 it's very rare!"
      "Are you interested?"
    ]

   if 'items.Valerium-Great-Sword "You are quite the master, I see! 
Care to sell your Valerium Great Sword?"
;