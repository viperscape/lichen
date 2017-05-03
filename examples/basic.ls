root
    # lichen starts with a root node by default
    # otherwise you can start the evaluator from a specific node-name

    emit "Welcome, let's head over to the information node"

    # tag next with 'now' to immediately advance
    next:now info # info specifies the node-name to head to
;

info
    # use brackets [] to use multiline lists
    # use a backtick to format the string with the environment variable
    emit ["Information node here"
         "The my-globals size var is `my-globals.size"]

    next:now end
;

# notice the order of nodes does not matter
def my-globals
    # define a local environment of variables to use
    size 5
;

end
    emit "Good bye!"

    # the evaluator stops when no more nodes will be reached
;