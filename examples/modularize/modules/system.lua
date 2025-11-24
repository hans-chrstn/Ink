local cfg = require("config")
return {
	type = "GtkBox",
	properties = {
		orientation = "vertical",
		spacing = cfg.spacing,
		margin_top = cfg.padding,
		margin_start = cfg.padding,
		margin_end = cfg.padding,
		valign = "start",
	},
	children = {
		{ type = "GtkLabel", properties = { label = "<b>System Interop</b>", use_markup = true, xalign = 0 } },
		{
			type = "GtkButton",
			properties = { label = "Get System Kernel (exec)" },
			signals = {
				clicked = function()
					local kernel = exec("uname -r")
					print("Kernel Version: " .. kernel)
				end,
			},
		},
		{
			type = "GtkButton",
			properties = { label = "Launch File Manager (spawn)" },
			signals = {
				clicked = function()
					spawn("xdg-open ~")
				end,
			},
		},
		{ type = "GtkSeparator", properties = { orientation = "horizontal" } },
		{ type = "GtkLabel", properties = { label = "<b>Network</b>", use_markup = true, xalign = 0 } },
		{
			type = "GtkButton",
			properties = { label = "Fetch Public IP (HTTP GET)" },
			signals = {
				clicked = function()
					local ip_data = fetch("https://api.ipify.org")
					print("My IP Address: " .. ip_data)
				end,
			},
		},
		{ type = "GtkSeparator", properties = { orientation = "horizontal" } },
		{ type = "GtkLabel", properties = { label = "<b>Visuals</b>", use_markup = true, xalign = 0 } },
		{
			type = "GtkBox",
			properties = { orientation = "horizontal", spacing = 20 },
			children = {
				{ type = "GtkSpinner", properties = { spinning = true } },
				{ type = "GtkColorButton", properties = { use_alpha = true } },
				{ type = "GtkFontButton", properties = { use_font = true } },
			},
		},
	},
}
