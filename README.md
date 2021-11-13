Rough proof of concept of multiple windows in egui

Includes opening more windows at runtime and accessing / mutating state between windows. See https://youtu.be/hHL9riM5ELQ

Currently only tested on mac and windows.

Inspired by / derived from mini_fb_gl's [multi_window example](https://github.com/shivshank/mini_gl_fb/blob/master/examples/multi_window.rs)


## known issues

if you open a bunch of popup windows and then close the root, the app might not exit and continues to run in the background.