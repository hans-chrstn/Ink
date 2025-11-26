local cfg = require("config")
local apps = Apps.list()

table.sort(apps, function(a, b)
	return a.name:lower() < b.name:lower()
end)

local app_grid_widget = nil

local function create_app_button(app)
	return {
		type = "GtkButton",
		properties = {
			valign = "start",
			has_frame = false,
			hexpand = true,
			visible = true,
		},
		children = {
			{
				type = "GtkBox",
				properties = { orientation = "vertical", spacing = 6 },
				children = {
					{ type = "GtkImage", properties = { icon_name = app.icon, pixel_size = cfg.icon_size } },
					{
						type = "GtkLabel",
						properties = { label = app.name, wrap = true, max_width_chars = 12, ellipsize = "end" },
					},
				},
			},
		},
		signals = {
			clicked = function()
				app:launch()
				exit()
			end,
		},
	}
end

local function filter_apps(text)
	if not app_grid_widget then
		return
	end

	app_grid_widget:remove_children()

	local query = text:lower()
	for _, app in ipairs(apps) do
		if app.name:lower():find(query, 1, true) then
			local btn = build_ui(create_app_button(app))
			app_grid_widget:add(btn)
		end
	end
end

return {
	type = "GtkApplicationWindow",
	window_mode = "layer_shell",
	layer = "overlay",
	anchors = { top = true, bottom = true, left = true, right = true },
	keyboard_mode = "exclusive",

	keymaps = {
		["Escape"] = function()
			exit()
		end,
	},

	css = [[
        window { background-color: rgba(0, 0, 0, 0.85); }
        button { padding: 12px; border-radius: 12px; }
        button:hover { background-color: rgba(255, 255, 255, 0.15); }
        label { color: white; font-weight: bold; }
        entry { background: rgba(255,255,255,0.1); color: white; border-radius: 8px; padding: 10px; }
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
			},
			children = {
				{
					type = "GtkSearchEntry",
					properties = { placeholder_text = "Type to search..." },
					signals = {
						search_changed = function(self)
							filter_apps(self:get_text())
						end,
						map = function(self)
							self:grab_focus()
						end,
					},
				},
				{
					type = "GtkScrolledWindow",
					properties = { hscrollbar_policy = "never", vexpand = true, min_content_height = 400 },
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
							signals = {
								map = function(self)
									app_grid_widget = self
									filter_apps("")
								end,
							},
						},
					},
				},
			},
		},
	},
}
