root
    @player.weight 50
    @player.name "Io"
    now town
;

store
    has_weight player.weight < 100


    if !player.visited_store "G'day, you look weary, `player.name"
    or "Welcome back my friend, `player.name"

    @player.visited_store true
    
    if !has_weight ["You are overloaded, `player.name"
       "Leaving store now"
       now town]

    if !player.items.Dragonscale-Great-Sword [
       "Let me tell you about the rare Dragonscale Great Sword"
       "Are you interested?"
       await info-dragonscale]

    if player.Dragonscale-Great-Sword "You are quite the master, I see!"

    emit "See you later!"
;

info-dragonscale
    emit ["There is a long history of Dragonscale"
         "It all started.."]
    @player.Dragonscale-Great-Sword new sword
    @player.weight + player.Dragonscale-Great-Sword.weight
    emit "Here have this great sword, made of Dragonscale"
;

town
    select {"Head to Store?" store,
            "Leave the town?" leave}

    emit "A dustball blows by"
    restart
;

leave
    emit "`player.name heads off into the sunset"
    exit
;

def sword
    damage 5
    weight 50
;