local cfg = require("config")

return {
	type = "GtkBox",
	properties = {
		orientation = "vertical",
		spacing = cfg.spacing,
		margin_top = cfg.margin,
		margin_start = cfg.margin,
		valign = "start",
	},
	children = {
		{ type = "GtkLabel", properties = { label = "<b>Inputs</b>", use_markup = true, xalign = 0 } },
		{ type = "GtkEntry", properties = { placeholder_text = "Username" } },
		{ type = "GtkPasswordEntry", properties = { placeholder_text = "Password" } },
		{ type = "GtkSpinButton", properties = { digits = 2, value = 50.0 } },
	},
}
