from kivy.app import App
from kivy.clock import Clock
from kivy.core.window import WindowBase, Window
from kivy.properties import (BooleanProperty, NumericProperty, ObjectProperty,
                             ReferenceListProperty)
from kivy.uix.widget import Widget
from kivy.vector import Vector

from springs import AnimatedValue, SpringConfig, step_spring

SPRING_CONFIG = SpringConfig()
SPRING_CONFIG.clamp = True

class CameraFocalPoint(Widget):
    emphasized = BooleanProperty(False)
    pan_value = AnimatedValue()
    tilt_value = AnimatedValue()

    def update(self, dt, new_target):
        if new_target != None:
            x, y = new_target
            self.pan_value.to_pos = x - self.width / 2
            self.tilt_value.to_pos = y - self.height / 2

        step_spring(dt, self.pan_value, SPRING_CONFIG)
        step_spring(dt, self.tilt_value, SPRING_CONFIG)

        self.pos = (self.pan_value.cur_pos, self.tilt_value.cur_pos)

    def color(self, emphasized):
        if emphasized:
            return [1, 0, 0, 1]
        else:
            return [1, 1, 1, 1]

class SmoothFollow(Widget):
    cursor_over_app = BooleanProperty(False)
    camera_focus = ObjectProperty(None)

    def update(self, dt):
        w: WindowBase = self.get_root_window()
        target = w.mouse_pos if self.cursor_over_app else None
        self.camera_focus.update(dt, new_target=target)

class SmoothFollowApp(App):

    def on_start(self):
        self.register_window_events()

    def register_window_events(self):
        w: WindowBase = self.root_window
        w.bind(on_cursor_enter = self.on_cursor_enter)
        w.bind(on_cursor_leave = self.on_cursor_leave)

    def on_cursor_enter(self, _):
        self.root.cursor_over_app = True
    def on_cursor_leave(self, _):
        self.root.cursor_over_app = False

    def build(self):
        root = SmoothFollow()
        Clock.schedule_interval(root.update, 1.0 / 60.0)
        return root



if __name__ == "__main__":
    SmoothFollowApp().run()
