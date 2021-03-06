# -*- coding: utf-8 -*-

import random
from os import system

from tav import core
from tav.tmux import hook


def test_update():
  hook.disable('test_upate')

  name = f'_tav_test_{random.randrange(20000)}_'

  cmdstr = f'''
    tmux new-session -s {name} -n window1 -d sh
  '''

  system(cmdstr)

  assert core.updateModel() is True
  assert core.updateModel() is False

  cmdstr = f'''
    tmux kill-session -t {name}
  '''

  system(cmdstr)

  assert core.updateModel() is True
  assert core.updateModel() is False

  hook.enable('after test_upate')
