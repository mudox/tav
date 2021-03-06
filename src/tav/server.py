#!/usr/bin/env python3
# -*- coding: utf-8 -*-

import logging
import re
import socket
import traceback
from contextlib import closing
from http.server import BaseHTTPRequestHandler, HTTPServer
from sys import exit

import daemon

from jaclog import jaclog

from . import settings as cfg
from . import core, tmux, watcher

logger = logging.getLogger(__name__)


def getFreePort():
  with closing(socket.socket(socket.AF_INET, socket.SOCK_STREAM)) as sock:
    sock.bind(('', 0))
    return sock.getsockname()[1]


def start():
  watcher.startMonitoring()

  logger.debug('spawn daemon ...')

  with daemon.DaemonContext():

    jaclog.configure(
        appName='tav', fileName='server.log', printSessionLine=False)

    try:
      port = 32323
      server = HTTPServer(('', port), HTTPRequestHandler)

    except OSError as e:
      if e.errno == 48:
        alternatePort = getFreePort()
        server = HTTPServer(('', alternatePort), HTTPRequestHandler)
        return
      else:
        raise
    except BaseException:
      logger.error(traceback.format_exc())
      raise

    logger.info(f'listening at localhost:{port}')
    cfg.paths.port.write_text(str(port))

    try:
      server.serve_forever()
    except BaseException:
      logger.error(traceback.format_exc())
      raise


class HTTPRequestHandler(BaseHTTPRequestHandler):

  server_version = 'Tav/0.1'
  protocol_version = 'HTTP/1.1'  # enable persistent connection
  error_content_type = 'text/plain'

  def do_GET(self):
    routes = [
        self._greet,
        self._attach,
        self._hook,
        self._event,
        self._stop,
    ]

    try:
      for route in routes:
        if route():
          return
      else:
        self._invalidPath()
    except BaseException:
      logger.error(traceback.format_exc())
      raise

  def log_message(self, format, *args):
    msg = format % tuple(args)
    logger.debug(f'🎃  {msg}')

  def _invalidPath(self):
    self.send_error(403, 'invalid path')

  def _sendBare200(self):
    self.send_response(200)
    self.send_header('Content-Length', 0)
    self.end_headers()

  #
  # nodes
  #

  def _event(self):
    m = re.match(r'^/event/([^/]+)/$', self.path)
    if m is None:
      return False

    else:
      self._sendBare200()

      event = m.group(1)
      core.onTmuxEvent(event)

      return True

  def _stop(self):
    if self.path != '/stop/':
      return False

    else:
      self._sendBare200()
      logger.info('server exit')
      exit()

  def _attach(self):
    if self.path != '/attach/':
      return False

    else:
      self._sendBare200()

      tmux.switchTo(cfg.tmux.yang.target)
      return True

  def _greet(self):
    if self.path != '/hello/tav/':
      return False

    else:
      self._sendBare200()
      return True

  def _hook(self):
    m = re.match(r'^/hook/([^/]+)/([^/]+)/$', self.path)
    if m is None:
      return False

    else:
      self._sendBare200()

      action = m.group(1)
      reason = m.group(2)
      if action == 'enable':
        tmux.hook.enable(reason)
      elif action == 'disable':
        tmux.hook.disable(reason)
      else:
        logger.warning(f'invalid path {self.path}')

      return True
