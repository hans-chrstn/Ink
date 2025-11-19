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
		{ type = "GtkLabel", properties = { label = "<b>Media</b>", use_markup = true, xalign = 0 } },
		{ type = "GtkSpinner", properties = { spinning = true, hexpand = true, vexpand = true } },
		{ type = "GtkLevelBar", properties = { value = 0.8 } },
	},
}
