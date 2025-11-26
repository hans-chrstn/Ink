local cfg = require("config")
local mod_basics = require("modules.basics")
local mod_containers = require("modules.containers")
local mod_system = require("modules.system")
return {
	type = "GtkApplicationWindow",
	window_mode = "layer_shell",
	layer = "top",
	keyboard_mode = "on_demand",
	anchors = { top = true, left = true, right = true, bottom = true },
	properties = {
		title = "Ink Feature Demo",
		default_width = cfg.win_width,
		default_height = cfg.win_height,
		visible = true,
	},
	children = {
		{
			type = "GtkBox",
			properties = { orientation = "vertical" },
			children = {
				{
					type = "GtkHeaderBar",
					properties = { show_title_buttons = true },
				},
				{
					type = "GtkNotebook",
					properties = {
						tab_pos = "top",
						vexpand = true,
					},
					children = {
						mod_basics,
						mod_containers,
						mod_system,
					},
				},
			},
		},
	},
}
