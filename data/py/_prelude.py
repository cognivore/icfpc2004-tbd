# true with probability 1/n
def flip(n):
    _output('flip ' + str(n) + ' 1 0')
    return _input() == 1


orientation = None  # set to 0 for orientation tracking

def turn_left():
    _output('turn left 0')
    _input()
    global orientation
    if orientation != None:
        orientation -= 1
        orientation %= 6

def turn_right():
    _output('turn right 0')
    _input()
    global orientation
    if orientation != None:
        orientation += 1
        orientation %= 6


HERE = 'here'
AHEAD = 'ahead'
LEFT_AHEAD = 'left_ahead'
RIGHT_AHEAD = 'right_ahead'

FRIEND = 'friend'
FOE = 'foe'
FRIEND_WITH_FOOD = 'friendwithfood'
FOE_WITH_FOOD = 'foewithfood'
FOOD = 'food'
ROCK = 'rock'
FOE_MARKER = 'foemarker'
HOME = 'home'
FOE_HOME = 'foehome'

def sense(dir, cond):
    _output('sense ' + dir + ' 1 0 ' + cond)
    return _input() == 1


def sense_marker(dir, marker_no):
    _output('sense ' + dir + ' 1 0 marker ' + str(marker_no))
    return _input() == 1


def pick_up():
    _output('pickup 1 0')
    return _input() == 1


def drop():
    _output('drop 0')
    _input()


def move():
    _output('move 1 0')
    return _input() == 1


def mark(marker_no):
    _output('mark ' + str(marker_no) + ' 0')
    _input()

def unmark(marker_no):
    _output('unmark ' + str(marker_no) + ' 0')
    _input()
