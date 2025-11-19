local config = require("config")
local inputs = require("components.inputs")
local buttons = require("components.buttons")
local media = require("components.media")

---@type WindowConfig
return {
	type = "GtkApplicationWindow",
	keyboard_mode = "on_demand",
	window_mode = "layer_shell",
	layer = "top",
	anchors = { top = true, left = true, right = true, bottom = true },

	properties = {
		title = "Ink Modular Test",
		default_width = 900,
		default_height = 600,
	},

	children = {
		{
			type = "GtkBox",
			properties = { orientation = "vertical" },
			children = {
				{ type = "GtkHeaderBar", properties = { show_title_buttons = true } },
				{
					type = "GtkNotebook",
					properties = { tab_pos = "top" },
					children = {
						buttons,
						inputs,
						media,
					},
				},
			},
		},
	},
}
