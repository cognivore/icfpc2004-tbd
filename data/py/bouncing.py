def turn_around():
    turn_left()
    turn_left()
    turn_left()


def random_turn():
    if flip(2):
        if flip(2):
            turn_left()
        else:
            turn_right()
    else:
        if flip(2):
            turn_left()
            turn_left()
        else:
            turn_right()
            turn_right()


has_food = False
while True:
    if has_food:
        if sense(HERE, HOME):
            drop()
            has_food = False
            turn_around()
    else:
        if sense(HERE, FOOD) and pick_up():
            has_food = True
            turn_around()

    if not move():
        random_turn()
