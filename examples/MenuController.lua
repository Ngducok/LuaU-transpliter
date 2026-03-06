local Players = game:GetService("Players")
local ReplicatedStorage = game:GetService("ReplicatedStorage")

local player = Players.LocalPlayer
local playerGui = player:WaitForChild("PlayerGui")

Controller = {}
function Controller.OnPlay()
    print("Play clicked!")
end

function Controller.OnSettings()
    print("Settings clicked!")
end

function Controller.OnQuit()
    print("Quit clicked!")
end

menuRoot.Parent = playerGui
menuRoot.Name = "MenuUI"
