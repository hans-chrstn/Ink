local cfg = require("config")
return {
	type = "GtkBox",
	properties = {
		orientation = "vertical",
		spacing = cfg.spacing,
		margin_top = cfg.padding,
		margin_start = cfg.padding,
		margin_end = cfg.padding,
		valign = "fill",
		vexpand = true,
	},
	children = {
		{ type = "GtkLabel", properties = { label = "<b>Complex Containers</b>", use_markup = true, xalign = 0 } },
		{
			type = "GtkExpander",
			properties = {
				label = "Click to Expand Details",
				expanded = false,
			},
			children = {
				{
					type = "GtkLabel",
					properties = {
						label = "Hidden content revealed!\nYou can put any widget here.",
						wrap = true,
					},
				},
			},
		},
		{ type = "GtkSeparator", properties = { orientation = "horizontal", margin_top = 10, margin_bottom = 10 } },
		{ type = "GtkLabel", properties = { label = "Split View (Paned)", xalign = 0 } },
		{
			type = "GtkPaned",
			properties = {
				orientation = "horizontal",
				position = 150,
				wide_handle = true,
				vexpand = true,
			},
			children = {
				{
					type = "GtkBox",
					properties = { orientation = "vertical", spacing = 5 },
					children = {
						{ type = "GtkLabel", properties = { label = "Sidebar" } },
						{ type = "GtkButton", properties = { label = "Item 1" } },
						{ type = "GtkButton", properties = { label = "Item 2" } },
					},
				},
				{
					type = "GtkScrolledWindow",
					properties = { has_frame = true, min_content_height = 200 },
					children = {
						{
							type = "GtkTextView",
							properties = {
								monospace = true,
								editable = true,
							},
						},
					},
				},
			},
		},
	},
}
