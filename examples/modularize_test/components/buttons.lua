local cfg = require("config")

return {
	type = "GtkBox",
	properties = {
		orientation = "vertical",
		spacing = cfg.spacing,
		margin_top = cfg.margin,
		margin_start = cfg.margin,
		margin_end = cfg.margin,
	},
	children = {
		{ type = "GtkLabel", properties = { label = "<b>Actions</b>", use_markup = true, xalign = 0 } },

		{
			type = "GtkButton",
			properties = { label = "Check Uptime (Exec)" },
			signals = {
				clicked = function()
					local out = exec("uptime")
					print("Uptime: " .. out)
				end,
			},
		},
		{
			type = "GtkButton",
			properties = { label = "Open Terminal (Spawn)" },
			signals = {
				clicked = function()
					spawn("wezterm")
				end,
			},
		},
		{
			type = "GtkSwitch",
			properties = { active = true, halign = "start" },
		},
	},
}
