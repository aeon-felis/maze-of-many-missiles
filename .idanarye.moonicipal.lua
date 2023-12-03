local moonicipal = require'moonicipal'
local T = moonicipal.tasks_file()
local P = require'idan.project.rust.bevy' {
    crate_name = 'maze_of_many_missiles',
    level_editor = true,
}
moonicipal.include(P)

function T:query()
    dump(P:cargo_required_features_for_all_examples())
end
