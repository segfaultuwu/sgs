sgs.window("bar", {
    layer = "top",
    anchor = "top",
    height = 36,
    class = { "bar" },

    child = sgs.centerbox({
        start = sgs.box({
            class = { "bar-left" },
            children = {
                sgs.button({
                    text = "SGS",
                    class = { "launcher" },
                    command = "rofi -show drun",
                }),

                sgs.box({
                    class = { "module", "workspace-module" },
                    children = {
                        sgs.workspaces({
                            count = 5,
                            class = { "workspaces" },
                            button_class = { "workspace-button" },
                        }),
                    },
                }),
            },
        }),

        center = sgs.box({
            class = { "bar-center" },
            children = {
                sgs.clock({
                    format = "%H:%M",
                    class = { "clock" },
                }),
            },
        }),

        ["end"] = sgs.box({
            class = { "bar-right" },
            children = {
                sgs.box({
                    class = { "module", "status-module" },
                    children = {
                        sgs.label({
                            text = "VOL",
                            class = { "volume" },
                        }),

                        sgs.label({
                            text = "BAT",
                            class = { "battery" },
                        }),
                    },
                }),

                sgs.button({
                    text = "⏻",
                    class = { "power" },
                    command = "wlogout",
                }),
            },
        }),
    }),
})
