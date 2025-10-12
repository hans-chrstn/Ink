-- ./example.lua
return {
  type = "GtkApplicationWindow",
  properties = {
    title = "My App",
    default_width = 100,
    default_height = 50
  },
  children = {
    {
      type = "GtkLabel",
      properties = {
        label = "Hello from Lua!"
      },
    },
  }
}
