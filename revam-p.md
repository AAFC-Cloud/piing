We are going to revamp the way the config is done.
right now we have "ConfigSnapshot" and we are storing some stuff in plaintext and some stuff as hcl files.

hcl stores a Body in a file, we want our config editing operations to operate on the stored hcl::edit Body structs so that we can programmatically update the user's code without breaking their formatting and decoration stuff.

This means our snapshot is effectively a 
{ 
    config_files: Map<PathBuf, Body>,
    decoded_adapter_criteria: Option<Vec<VpnCriterionProperties>>,
    decoded_targets: Option<Vec<Target>>
}
where we have helper fns on the snapshot to get the adapter_criteria() and decode if haven't done yet

we want to change our terminology from "host" to "target"
we will change the config from being a folder structure + plaintext files to having the config dir be a flat dir of .piing_hcl files

here is a demonstration

resource "piing_vpn_criterion" "Eddie" {
  display_name = "Eddie"
}

resource "piing_target" "google_dns" {
    value = "8.8.8.8"
    interval = "1s"
    mode = "icmp"
}

use git mv as necessary
we will change the cli from host to target terminology as well

we basically have

domain models like VpnCriterionProperties (which should just be renamed to VpnCriterion and we will remove the old VpnCriterion and VpnCriteria structs since the existing VpnCriterion `name` field only exists for our current half-assed write-to-hcl behaviour)

basically, the snapshot stores the actual files, we have fields for the decoded part that is only relevant for the actual runtime behaviour, and any modifications to the config has to modify the Body values in the map and invalidate the cache

the `piing target add 8.8.8.8 --mode icmp --interval 1s` command would create the PiingTarget { ... } struct, which would have a From<PiingTarget> for Body impl, and we would add it to the config hcl dict as a new file named after the date and time and we would write it to disk ensuring no clobbering; this makes our cli-generated values low conflict due to new file because of unique datetime filename, while letting us parse and not destroy the manually written config

if we get

`piing target remove "google_dns"`
then we will visit the bodies to mutate the body in place and write them back to disk; we can focus on the addition first and have a stubbed `target remove` before moving on to work on that part.

This is going to be a significant change to the codebase.
We will be going from 0.4 to 0.5, a breaking change, and we need zero backwards compatibility so we may make any change we wish.

Please overhaul the config and target commands.

Let's also add a `piing home` command that prints the home dir.