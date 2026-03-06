local Players = game:GetService("Players")
local player = Players.LocalPlayer
local playerGui = player:WaitForChild("PlayerGui")

Controller = {}
function Controller.OnPlay()
    -- TODO: add your logic
end

function Controller.OnSettings()
    -- TODO: add your logic
end

function Controller.OnCredits()
    -- TODO: add your logic
end

local root = Instance.new("ScreenGui")
root.Name = "UI"
root.ResetOnSpawn = false
root.IgnoreGuiInset = false

local frame_1 = Instance.new("Frame")
frame_1.Parent = root
frame_1.Name = "body_1"
frame_1.Size = UDim2.new(1, 0, 1, 0)
    local frame_2 = Instance.new("Frame")
    frame_2.Parent = frame_1
    frame_2.Name = "root"
    frame_2.Size = UDim2.new(1, 0, 1, 0)
    local uilistlayout_3 = Instance.new("UIListLayout")
    uilistlayout_3.Parent = frame_2
    uilistlayout_3.FillDirection = Enum.FillDirection.Horizontal
    uilistlayout_3.HorizontalAlignment = Enum.HorizontalAlignment.Center
    uilistlayout_3.VerticalAlignment = Enum.VerticalAlignment.Center
        local frame_4 = Instance.new("Frame")
        frame_4.Parent = frame_2
        frame_4.Name = "div_3"
        frame_4.Size = UDim2.new(0, 420, 0, 340)
        frame_4.BackgroundColor3 = Color3.fromHex("0f0f1a")
        frame_4.BorderSizePixel = 0
        local uilistlayout_5 = Instance.new("UIListLayout")
        uilistlayout_5.Parent = frame_4
        uilistlayout_5.HorizontalAlignment = Enum.HorizontalAlignment.Left
        uilistlayout_5.VerticalAlignment = Enum.VerticalAlignment.Top
        uilistlayout_5.FillDirection = Enum.FillDirection.Vertical
        local uicorner_6 = Instance.new("UICorner")
        uicorner_6.Parent = frame_4
        uicorner_6.CornerRadius = UDim.new(0, 20)
        local uipadding_7 = Instance.new("UIPadding")
        uipadding_7.Parent = frame_4
        uipadding_7.PaddingLeft = UDim.new(0, 40)
        uipadding_7.PaddingTop = UDim.new(0, 40)
        uipadding_7.PaddingRight = UDim.new(0, 40)
        uipadding_7.PaddingBottom = UDim.new(0, 40)
        frame_4.LayoutOrder = 1
            local textlabel_8 = Instance.new("TextLabel")
            textlabel_8.Parent = frame_4
            textlabel_8.Name = "h1_4"
            textlabel_8.LayoutOrder = 1
            textlabel_8.Text = "Welcome to Game"
            textlabel_8.TextColor3 = Color3.fromHex("ffffff")
            textlabel_8.TextXAlignment = Enum.TextXAlignment.Center
            textlabel_8.AutomaticSize = Enum.AutomaticSize.Y
            textlabel_8.TextSize = 32
            textlabel_8.RichText = true
            local textlabel_9 = Instance.new("TextLabel")
            textlabel_9.Parent = frame_4
            textlabel_9.Name = "p_5"
            textlabel_9.TextXAlignment = Enum.TextXAlignment.Center
            textlabel_9.LayoutOrder = 2
            textlabel_9.AutomaticSize = Enum.AutomaticSize.Y
            textlabel_9.TextColor3 = Color3.fromHex("8892b0")
            textlabel_9.Text = "Choose an option below"
            textlabel_9.RichText = true
            textlabel_9.TextSize = 16
            local frame_10 = Instance.new("Frame")
            frame_10.Parent = frame_4
            frame_10.Name = "div_6"
            local uilistlayout_11 = Instance.new("UIListLayout")
            uilistlayout_11.Parent = frame_10
            uilistlayout_11.HorizontalAlignment = Enum.HorizontalAlignment.Left
            uilistlayout_11.FillDirection = Enum.FillDirection.Vertical
            uilistlayout_11.Padding = UDim.new(0, 12)
            uilistlayout_11.VerticalAlignment = Enum.VerticalAlignment.Top
            local uipadding_12 = Instance.new("UIPadding")
            uipadding_12.Parent = frame_10
            uipadding_12.PaddingLeft = UDim.new(0, 0)
            uipadding_12.PaddingRight = UDim.new(0, 0)
            uipadding_12.PaddingTop = UDim.new(0, 0)
            uipadding_12.PaddingBottom = UDim.new(0, 0)
            frame_10.AutomaticSize = Enum.AutomaticSize.Y
            frame_10.LayoutOrder = 3
                local textbutton_13 = Instance.new("TextButton")
                textbutton_13.Parent = frame_10
                textbutton_13.Name = "button_7"
                textbutton_13.BackgroundColor3 = Color3.fromHex("6366f1")
                textbutton_13.BorderSizePixel = 0
                local uicorner_14 = Instance.new("UICorner")
                uicorner_14.Parent = textbutton_13
                uicorner_14.CornerRadius = UDim.new(0, 10)
                local uipadding_15 = Instance.new("UIPadding")
                uipadding_15.Parent = textbutton_13
                uipadding_15.PaddingTop = UDim.new(0, 14)
                uipadding_15.PaddingBottom = UDim.new(0, 28)
                uipadding_15.PaddingLeft = UDim.new(0, 14)
                uipadding_15.PaddingRight = UDim.new(0, 28)
                textbutton_13.AutomaticSize = Enum.AutomaticSize.Y
                textbutton_13.RichText = true
                textbutton_13.TextSize = 17
                textbutton_13.TextColor3 = Color3.fromHex("ffffff")
                textbutton_13.LayoutOrder = 1
                textbutton_13.Text = "Play"
                textbutton_13.TextXAlignment = Enum.TextXAlignment.Center
                textbutton_13.Activated:Connect(function()
                        if Controller["OnPlay"] then
                            Controller["OnPlay"]()
                        end
                end)
                do
                        local TweenService = game:GetService("TweenService")
                        local tweenInfo = TweenInfo.new(0.3, Enum.EasingStyle.Quad, Enum.EasingDirection.Out)
                        textbutton_13.AncestryChanged:Once(function()
                                if textbutton_13.Parent then
                                    textbutton_13.BackgroundTransparency = 1
                                    local tween = TweenService:Create(textbutton_13, tweenInfo, { BackgroundTransparency = 0 })
                                    tween:Play()
                                end
                        end)
                end
                local textbutton_16 = Instance.new("TextButton")
                textbutton_16.Parent = frame_10
                textbutton_16.Name = "button_8"
                textbutton_16.BackgroundColor3 = Color3.fromHex("1e1e2e")
                textbutton_16.BorderSizePixel = 0
                local uicorner_17 = Instance.new("UICorner")
                uicorner_17.Parent = textbutton_16
                uicorner_17.CornerRadius = UDim.new(0, 10)
                local uipadding_18 = Instance.new("UIPadding")
                uipadding_18.Parent = textbutton_16
                uipadding_18.PaddingBottom = UDim.new(0, 28)
                uipadding_18.PaddingTop = UDim.new(0, 14)
                uipadding_18.PaddingRight = UDim.new(0, 28)
                uipadding_18.PaddingLeft = UDim.new(0, 14)
                textbutton_16.Text = "Settings"
                textbutton_16.TextXAlignment = Enum.TextXAlignment.Center
                textbutton_16.AutomaticSize = Enum.AutomaticSize.Y
                textbutton_16.LayoutOrder = 2
                textbutton_16.RichText = true
                textbutton_16.TextColor3 = Color3.fromHex("e2e8f0")
                textbutton_16.TextSize = 17
                textbutton_16.Activated:Connect(function()
                        if Controller["OnSettings"] then
                            Controller["OnSettings"]()
                        end
                end)
                local textbutton_19 = Instance.new("TextButton")
                textbutton_19.Parent = frame_10
                textbutton_19.Name = "button_9"
                textbutton_19.BackgroundColor3 = Color3.fromHex("1e1e2e")
                textbutton_19.BorderSizePixel = 0
                local uicorner_20 = Instance.new("UICorner")
                uicorner_20.Parent = textbutton_19
                uicorner_20.CornerRadius = UDim.new(0, 10)
                local uipadding_21 = Instance.new("UIPadding")
                uipadding_21.Parent = textbutton_19
                uipadding_21.PaddingTop = UDim.new(0, 14)
                uipadding_21.PaddingRight = UDim.new(0, 28)
                uipadding_21.PaddingLeft = UDim.new(0, 14)
                uipadding_21.PaddingBottom = UDim.new(0, 28)
                textbutton_19.TextXAlignment = Enum.TextXAlignment.Center
                textbutton_19.RichText = true
                textbutton_19.Text = "Credits"
                textbutton_19.AutomaticSize = Enum.AutomaticSize.Y
                textbutton_19.TextSize = 17
                textbutton_19.TextColor3 = Color3.fromHex("e2e8f0")
                textbutton_19.LayoutOrder = 3
                textbutton_19.Activated:Connect(function()
                        if Controller["OnCredits"] then
                            Controller["OnCredits"]()
                        end
                end)

root.Parent = playerGui
