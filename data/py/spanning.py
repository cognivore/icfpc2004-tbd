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


orientation = 0
target_dir = None
while True:
    # no food
    while True:
        if (not sense_marker(HERE, 0) and
            not sense_marker(HERE, 1) and
            not sense_marker(HERE, 2)):
            if (orientation + 1) % 2 == 1:
                mark(0)
            if (orientation + 1) / 2 % 2 == 1:
                mark(1)
            if (orientation + 1) / 4 == 1:
                mark(2)

        if sense(HERE, FOOD) and not sense(HERE, HOME) and pick_up():
            break

        if not move():
            random_turn()

    # have food
    while True:
        if sense(HERE, HOME):
            drop()
            turn_left()
            turn_left()
            turn_left()
            break

        target_dir = 0-1 + 3
        if sense_marker(HERE, 0):
            target_dir += 1
        if sense_marker(HERE, 1):
            target_dir += 2
        if sense_marker(HERE, 2):
            target_dir += 4
        target_dir %= 6

        while orientation != target_dir:
            turn_right()
        target_dir = None
        move()
