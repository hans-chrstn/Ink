local cfg = require("config")
local volume_slider = nil
local revealer = nil
local hide_timer = nil
local ignore_update = false
local is_hovered = false
local function try_hide()
	if is_hovered then
		hide_timer = set_timeout(500, try_hide)
		return
	end
	if revealer then
		revealer:set_property("reveal_child", false)
	end
	hide_timer = nil
end
local function show_osd()
	if revealer then
		revealer:set_property("reveal_child", true)
	end
	if hide_timer then
		clearTimeout(hide_timer)
	end
	hide_timer = set_timeout(2000, try_hide)
end
local function sync_volume()
	local vol = Audio.get_volume()
	if volume_slider then
		ignore_update = true
		volume_slider:set_value(vol)
		ignore_update = false
	end
	show_osd()
end
return {
	type = "GtkApplicationWindow",
	window_mode = "layer_shell",
	layer = "overlay",
	anchors = { bottom = true, left = false, right = false, top = false },
	margins = { bottom = 100 },
	properties = {
		visible = true,
	},
	css = [[
        window { background-color: transparent; }
        .osd-box { 
            background-color: rgba(0,0,0,0.8); 
            border-radius: 20px; 
            padding: 20px; 
            color: white;
        }
        scale trough { min-height: 6px; background-color: rgba(255,255,255,0.2); border-radius: 3px; }
        scale highlight { background-color: #3584e4; border-radius: 3px; }
        scale slider { min-width: 20px; min-height: 20px; background-color: white; border-radius: 50%; }
    ]],
	children = {
		{
			type = "GtkRevealer",
			properties = {
				reveal_child = false,
				transition_type = "crossfade",
				transition_duration = 500,
			},
			signals = {
				map = function(self)
					revealer = self
					self:add_controller_motion(function()
						is_hovered = true
						show_osd()
					end, function()
						is_hovered = false
						show_osd()
					end)
					Audio.watch(function()
						sync_volume()
					end)
					sync_volume()
				end,
			},
			children = {
				{
					type = "GtkBox",
					properties = {
						orientation = "vertical",
						spacing = 10,
						width_request = 250,
						css_classes = { "osd-box" },
					},
					children = {
						{
							type = "GtkBox",
							properties = { orientation = "horizontal", spacing = 10 },
							children = {
								{
									type = "GtkImage",
									properties = { icon_name = "audio-volume-high", pixel_size = 24 },
								},
								{ type = "GtkLabel", properties = { label = "<b>Volume</b>", use_markup = true } },
							},
						},
						{
							type = "GtkScale",
							properties = {
								orientation = "horizontal",
								draw_value = false,
								has_origin = true,
							},
							signals = {
								map = function(self)
									volume_slider = self
									self:set_range(0, 100)
									self:set_increments(1, 10)
								end,
								value_changed = function(self)
									if ignore_update then
										return
									end
									is_hovered = true
									local val = self:get_value()
									Audio.set_volume(math.floor(val))
									show_osd()
								end,
							},
						},
					},
				},
			},
		},
	},
}
