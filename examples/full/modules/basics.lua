local cfg = require("config")

return {
	type = "GtkBox",
	properties = {
		orientation = "vertical",
		spacing = cfg.spacing,
		margin_top = cfg.padding,
		margin_bottom = cfg.padding,
		margin_start = cfg.padding,
		margin_end = cfg.padding,
		valign = "start",
	},
	children = {
		{ type = "GtkLabel", properties = { label = "<b>Basic Inputs</b>", use_markup = true, xalign = 0 } },

		{ type = "GtkEntry", properties = { placeholder_text = "Type text here..." } },

		{ type = "GtkPasswordEntry", properties = { placeholder_text = "Secret Password", show_peek_icon = true } },

		{ type = "GtkSearchEntry", properties = { placeholder_text = "Search..." } },

		{ type = "GtkSeparator", properties = { orientation = "horizontal" } },

		{
			type = "GtkBox",
			properties = { orientation = "horizontal", spacing = 10 },
			children = {
				{ type = "GtkSwitch", properties = { active = true, valign = "center" } },
				{ type = "GtkLabel", properties = { label = "Enable Turbo Mode" } },
			},
		},

		{ type = "GtkCheckButton", properties = { label = "I agree to the terms", active = false } },

		{ type = "GtkSeparator", properties = { orientation = "horizontal" } },

		{ type = "GtkLabel", properties = { label = "Volume", xalign = 0 } },
		{
			type = "GtkScale",
			properties = {
				orientation = "horizontal",
				draw_value = true,
				digits = 0,
				value_pos = "right",
				has_origin = true,
			},
		},

		{ type = "GtkLabel", properties = { label = "Download Progress", xalign = 0 } },
		{
			type = "GtkProgressBar",
			properties = {
				fraction = 0.75,
				show_text = true,
				text = "75%",
			},
		},

		{ type = "GtkLevelBar", properties = { value = 0.4, max_value = 1.0 } },
	},
}
