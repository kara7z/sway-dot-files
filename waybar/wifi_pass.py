#!/usr/bin/env python3
import sys
import gi
gi.require_version('Gtk', '3.0')
from gi.repository import Gtk, Gdk

class PasswordDialog(Gtk.Dialog):
    def __init__(self, ssid):
        super().__init__(title=f"Password for {ssid}")
        self.set_default_size(300, 100)
        self.set_decorated(False)
        self.set_keep_above(True)
        self.set_position(Gtk.WindowPosition.CENTER)

        css = b"""
        window {
            background-color: rgba(0, 0, 0, 0.85);
            border: 1px solid white;
        }
        label {
            color: white;
            font-family: "FiraCode Nerd Font";
            font-size: 16px;
        }
        entry {
            background-color: rgba(255, 255, 255, 0.1);
            color: white;
            border: 1px solid white;
            border-radius: 0;
            font-family: "FiraCode Nerd Font";
            font-size: 16px;
            padding: 8px;
        }
        button {
            background-color: rgba(255, 255, 255, 0.15);
            color: white;
            border: 1px solid white;
            border-radius: 0;
            font-family: "FiraCode Nerd Font";
            font-size: 14px;
            padding: 6px 16px;
            margin: 4px;
        }
        button:hover {
            background-color: rgba(255, 255, 255, 0.3);
            box-shadow: inset 0 -2px white;
        }
        """
        style_provider = Gtk.CssProvider()
        style_provider.load_from_data(css)
        Gtk.StyleContext.add_provider_for_screen(
            Gdk.Screen.get_default(),
            style_provider,
            Gtk.STYLE_PROVIDER_PRIORITY_APPLICATION
        )

        box = self.get_content_area()
        box.set_spacing(10)
        box.set_margin_start(15)
        box.set_margin_end(15)
        box.set_margin_top(10)
        box.set_margin_bottom(10)

        label = Gtk.Label(label=f"Password for {ssid}")
        box.add(label)

        self.entry = Gtk.Entry()
        self.entry.set_visibility(False)
        self.entry.set_input_purpose(Gtk.InputPurpose.PASSWORD)
        self.entry.connect("activate", self.on_connect)
        box.add(self.entry)

        btn_box = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL, spacing=10)
        cancel_btn = Gtk.Button(label="Cancel")
        cancel_btn.connect("clicked", self.on_cancel)
        btn_box.pack_start(cancel_btn, True, True, 0)

        connect_btn = Gtk.Button(label="Connect")
        connect_btn.connect("clicked", self.on_connect)
        btn_box.pack_start(connect_btn, True, True, 0)
        box.add(btn_box)

        self.entry.grab_focus()
        self.show_all()

    def on_connect(self, widget):
        self.password = self.entry.get_text()
        self.response(Gtk.ResponseType.OK)
        self.destroy()

    def on_cancel(self, widget):
        self.password = ""
        self.response(Gtk.ResponseType.CANCEL)
        self.destroy()

if __name__ == "__main__":
    ssid = sys.argv[1] if len(sys.argv) > 1 else "WiFi"
    dialog = PasswordDialog(ssid)
    dialog.password = ""
    response = dialog.run()
    if response == Gtk.ResponseType.OK and dialog.password:
        print(dialog.password)
    try:
        dialog.destroy()
    except Exception:
        pass
