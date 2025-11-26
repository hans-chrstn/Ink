local cfg = require("config")
local is_green = false
return {
	type = "GtkApplicationWindow",
	properties = {
		default_width = 500,
		default_height = 700,
		visible = true,
	},
	css = [[
        .green-button { background: #2ec27e; color: white; }
        .red-button { background: #e01b24; color: white; }
        label { font-size: 16px; }
    ]],
	children = {
		{
			type = "GtkBox",
			properties = {
				orientation = "vertical",
				spacing = 12,
				margin_top = 20,
				margin_bottom = 20,
				margin_start = 20,
				margin_end = 20,
			},
			children = {
				{
					type = "GtkLabel",
					properties = { label = "<b>1. Grid Layout Test</b>", use_markup = true, xalign = 0 },
				},
				{
					type = "GtkGrid",
					properties = { row_spacing = 10, column_spacing = 10, hexpand = true, halign = "fill" },
					children = {
						{
							type = "GtkButton",
							properties = { label = "Col 0, Row 0", grid_col = 0, grid_row = 0, hexpand = true },
						},
						{
							type = "GtkButton",
							properties = { label = "Col 1, Row 0", grid_col = 1, grid_row = 0, hexpand = true },
						},
						{
							type = "GtkButton",
							properties = {
								label = "Wide Item (Span 2)",
								grid_col = 0,
								grid_row = 1,
								grid_width = 2,
								hexpand = true,
							},
						},
					},
				},
				{ type = "GtkSeparator", properties = { orientation = "horizontal" } },
				{
					type = "GtkLabel",
					properties = { label = "<b>2. Interactive Widgets (Self)</b>", use_markup = true, xalign = 0 },
				},
				{
					type = "GtkButton",
					properties = { label = "Click to turn Green" },
					signals = {
						clicked = function(self)
							if is_green then
								self:remove_class("green-button")
								self:set_property("label", "Click to turn Green")
								is_green = false
							else
								self:add_class("green-button")
								self:set_property("label", "I am Green Now!")
								is_green = true
							end
						end,
					},
				},
				{
					type = "GtkSwitch",
					properties = { active = false, valign = "center" },
					signals = {
						state_set = function(self, state)
							if state then
								self:add_class("red-button")
							else
								self:remove_class("red-button")
							end
							return true
						end,
					},
				},
				{ type = "GtkSeparator", properties = { orientation = "horizontal" } },
				{
					type = "GtkLabel",
					properties = { label = "<b>3. Async &amp; Services</b>", use_markup = true, xalign = 0 },
				},
				{
					type = "GtkButton",
					properties = { label = "Check Battery & Wifi" },
					signals = {
						clicked = function(self)
							local success, cap, status = pcall(System.get_battery)
							local battery_info_str
							if success and cap ~= nil then
								battery_info_str = "Battery: " .. cap .. "% (" .. status .. ")"
							else
								battery_info_str = "No Battery Found" -- Simplified message
							end
							self:set_property("label", battery_info_str)
							exec_async("nmcli -t -f active,ssid dev wifi | grep '^yes' | cut -d: -f2", function(ssid)
								print("Wifi SSID: " .. ssid)
							end)
						end,
					},
				},
				{
					type = "GtkButton",
					properties = { label = "Test HTTP Fetch" },
					signals = {
						clicked = function()
							fetch_async("GET", "https://api.ipify.org", nil, nil, function(result)
								if result.ok then
									print("My IP: " .. tostring(result.ok))
								else
									print("Fetch Error: " .. tostring(result.err))
								end
							end)
						end,
					},
				},
				{
					type = "GtkBox",
					properties = { orientation = "horizontal" },
					children = {
						{
							type = "GtkButton",
							properties = { label = "Vol Up" },
							signals = {
								clicked = function()
									Audio.set_volume(Audio.get_volume() + 5)
								end,
							},
						},
						{
							type = "GtkButton",
							properties = { label = "Vol Down" },
							signals = {
								clicked = function()
									Audio.set_volume(Audio.get_volume() - 5)
								end,
							},
						},
					},
				},
			},
		},
	},
}
