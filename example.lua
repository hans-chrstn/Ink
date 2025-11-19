---@type WindowConfig
return {
	type = "GtkApplicationWindow",

	window_mode = "layer_shell",
	layer = "top",
	anchors = {
		top = true,
		left = true,
		right = true,
		bottom = false,
	},
	margins = {
		top = 10,
		left = 10,
		right = 10,
	},
	auto_exclusive_zone = true,

	properties = {
		title = "My Ink Bar",
		default_height = 40,
		css_classes = { "my-window" },
	},

	children = {
		{
			type = "GtkBox",
			properties = {
				orientation = "horizontal",
				spacing = 12,
				hexpand = true,
			},
			children = {
				{
					type = "GtkLabel",
					properties = {
						label = "<b>Ink</b> System",
						use_markup = true,
					},
				},
				{
					type = "GtkButton",
					properties = {
						label = "Click Me",
					},
					signals = {
						clicked = function()
							print("Button was clicked!")
						end,
					},
				},
				{
					type = "GtkButton",
					properties = {
						label = "Exit",
					},
					signals = {
						clicked = function()
							print("Exit clicked")
						end,
					},
				},
			},
		},
	},
}
