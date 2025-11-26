local cfg = require("config")
local apps = {}
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
	print("DEBUG (Lua): filter_apps called with text: '" .. text .. "'")
	if not app_grid_widget then
		print("DEBUG (Lua): filter_apps called but app_grid_widget is nil. Cannot filter.")
		return
	end

	app_grid_widget:remove_children()

	local query = text:lower()
	local added_count = 0
	for _, app_item in ipairs(apps) do
		if app_item.name:lower():find(query, 1, true) then
			local btn = build_ui(create_app_button(app_item))
			if btn then
				app_grid_widget:add(btn)
				added_count = added_count + 1
			else
				print("DEBUG (Lua): Failed to build UI for app: " .. app_item.name)
			end
		end
	end
	print("DEBUG (Lua): Filtered and added " .. added_count .. " buttons for query '" .. text .. "'.")
end

function app.on_ready()
	print("DEBUG (Lua): STARTING on_ink_ready function.")
	apps = Apps.list()
	print("DEBUG (Lua): Apps.list() returned. Found " .. #apps .. " applications.")
	table.sort(apps, function(a, b)
		return a.name:lower() < b.name:lower()
	end)

	for i = 1, math.min(#apps, 3) do
		print("DEBUG (Lua): App " .. i .. ": Name = " .. apps[i].name .. ", Icon = " .. apps[i].icon)
	end

	app_grid_widget = app.get_widget_by_id("app_grid")
	if app_grid_widget then
		print("DEBUG (Lua): app_grid_widget ('app_grid') found via app.get_widget_by_id. Filtering apps...")
		filter_apps("")
	else
		print("DEBUG (Lua): app_grid_widget ('app_grid') NOT found via app.get_widget_by_id! This is a critical error.")
	end
	print("DEBUG (Lua): on_ink_ready function finished.")
end

return {
	type = "GtkApplicationWindow",
	window_mode = "layer_shell",
	layer = "overlay",
	anchors = { top = true, bottom = true, left = true, right = true },
	keyboard_mode = "exclusive",
	properties = {
		visible = true,
	},
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
							id = "app_grid",
							properties = {
								valign = "start",
								homogeneous = true,
								min_children_per_line = 6,
								max_children_per_line = 6,
								row_spacing = 20,
								column_spacing = 20,
								selection_mode = "none",
							},
						},
					},
				},
			},
		},
	},
}
