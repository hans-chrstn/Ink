local cfg = require("config")

local apps = Apps.list()

table.sort(apps, function(a, b)
	return a.name:lower() < b.name:lower()
end)

local app_buttons = {}

for _, app in ipairs(apps) do
	table.insert(app_buttons, {
		type = "GtkButton",
		properties = {
			valign = "start",
			has_frame = false,
			hexpand = true,
		},
		children = {
			{
				type = "GtkBox",
				properties = {
					orientation = "vertical",
					spacing = 6,
				},
				children = {
					{
						type = "GtkImage",
						properties = {
							icon_name = app.icon,
							pixel_size = cfg.icon_size,
						},
					},
					{
						type = "GtkLabel",
						properties = {
							label = app.name,
							wrap = true,
							max_width_chars = 12,
							ellipsize = "end",
							justify = "center",
						},
					},
				},
			},
		},
		signals = {
			clicked = function()
				print("🚀 Launching: " .. app.name)
				Apps.launch(app.executable)
			end,
		},
	})
end

return {
	type = "GtkApplicationWindow",
	window_mode = "layer_shell",
	layer = "overlay",
	anchors = { top = true, bottom = true, left = true, right = true },
	keyboard_mode = "on_demand",

	css = [[
        window { background-color: rgba(0, 0, 0, 0.8); }
        button { padding: 12px; border-radius: 12px; }
        button:hover { background-color: rgba(255, 255, 255, 0.1); }
        label { color: white; font-size: 12px; }
    ]],

	children = {
		{
			type = "GtkBox",
			properties = {
				orientation = "vertical",
				spacing = 20,
				halign = "center",
				valign = "center",
				width_request = cfg.win_width,
				height_request = cfg.win_height,
				margin_top = 50,
				margin_bottom = 50,
			},
			children = {
				{
					type = "GtkSearchEntry",
					properties = { placeholder_text = "Search applications..." },
				},
				{
					type = "GtkScrolledWindow",
					properties = {
						hscrollbar_policy = "never",
						vexpand = true,
						min_content_height = 400,
					},
					children = {
						{
							type = "GtkFlowBox",
							properties = {
								valign = "start",
								homogeneous = true,
								min_children_per_line = 6,
								max_children_per_line = 6,
								row_spacing = 20,
								column_spacing = 20,
								selection_mode = "none",
							},
							children = app_buttons,
						},
					},
				},
			},
		},
	},
}
