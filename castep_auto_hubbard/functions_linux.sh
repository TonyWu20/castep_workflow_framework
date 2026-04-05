# castep_command_u="qsub hpc.pbs_AU.sh"
# castep_command_alpha="qsub hpc.pbs_HU.sh"
#

function job_type_input {
	if [[ $1 == '' ]]; then
		read -r -e -p "job type: u/alpha?" job_type
	else
		job_type=$1
	fi

	until [[ $job_type == 'u' || $job_type == 'U' || $job_type == 'alpha' || $job_type == 'Alpha' ]]; do
		echo "Invalid job_type input: $job_type"
		read -r -e -p "Please input job type correctly as follows: u/U or alpha/Alpha?" job_type
	done
	case $job_type in
	u | U) job_type="u" ;;
	alpha | Alpha) job_type="alpha" ;;
	*)
		echo "Invalid job type input; ends program"
		exit 1
		;;
	esac

	input_job_type="$job_type"
}

function setup {
	init_hubbard_u=$1
	init_elec_energy_tol=$2
	init_input_u=$3
	u_step=$4
	final_U=$5
	job_type_input "$6"
	job_type=$input_job_type
}

function setup_new_seed_folder {
	# Set new folder inside the $SEED_PATH - 2025-06-06
	# Current position: auto_hubbard_linux.sh location
	local new_seed_path="$SEED_PATH"_"$job_type"_"$init_input_u"_"$u_step"_"$final_U"_"$PERTURB_INIT_ALPHA"_"$PERTURB_INCREMENT"_"$PERTURB_FINAL_ALPHA"_STEPS_"$PERTURB_TIMES"
	echo "New directory: $new_seed_path"
	cd "$SEED_PATH" || exit 1
	# cd into $SEED_PATH
	echo "setup_new_seed_folder:${pwd}"
	if [ ! -d $new_seed_path ]; then
		mkdir $new_seed_path
	fi
	echo "Use 'find' to filter out result files and directory and then copy to new folder"
	find . -maxdepth 1 -type f -not -name "*.castep" -not -name "*.txt" -not -name "*.csv" -not -name "*.xsd" -not -name "*.xms" |
		xargs -I {} cp {} "$new_seed_path"/{}
	# Set the next $SEED_PATH to this created_folder as the base.
	SEED_PATH=$new_seed_path
	log_path=$(create_log "$job_type")
	# create a new final datasheet every time
	printf "Jobname,Channel ID,Spin,Before SCF,1st SCF,Last SCF,Converged\n" >"$SEED_PATH"/result_"$job_type"_final.csv
	# create a datasheet for instant recording
	printf "Jobname,Channel ID,Spin,Before SCF,1st SCF,Last SCF,Converged\n" >"$SEED_PATH"/result_"$job_type".csv
	# position: $SEED_PATH
}

function setup_perturbation {
	PERTURB_INIT_ALPHA=$1
	if [[ $PERTURB_INIT_ALPHA == '' ]]; then
		PERTURB_INIT_ALPHA=0.05
	fi
	PERTURB_INCREMENT=$2
	if [[ $PERTURB_INCREMENT == '' ]]; then
		PERTURB_INCREMENT=0.05
	fi
	PERTURB_FINAL_ALPHA=$3
	if [[ $PERTURB_FINAL_ALPHA == '' ]]; then
		PERTURB_FINAL_ALPHA=0.25
	fi
}

function setup_castep_command {
	castep_command_u=$1
	castep_command_alpha=$2
	castep_program_u_path="$script_path"/"$3"
	castep_program_alpha_path="$script_path"/"$4"
}

function faux_castep_run {
	sleep 1
	touch "$1.castep"
	sleep 1
	{
		echo "           1           1 Total:    4.88454712510949       Mz:"
		echo "           1           2 Total:    2.04480063601341       Mz:"
		echo "           1           1 Total:    4.88222767868911       Mz:"
		echo "           1           2 Total:    2.03625090967629       Mz:"
		echo "           1           1 Total:    4.88417863385162       Mz:"
		echo "           1           2 Total:    2.03022846329140       Mz:"
	} >>"$1.castep"
	sleep 1
	echo "Finalisation time" >>"$1.castep"
}

function hubbard_before {
	local init_hubbard_u=$1
	local i=$2
	local job_type=$3
	local U_value
	local alpha_value
	local i_u_value
	i_u_value=$(echo "$init_hubbard_u $i" | awk '{printf "%.14f0", $1+$2}')
	# Replace original u settings
	case $job_type in
	u)
		U_value=$i_u_value
		alpha_value=$init_hubbard_u
		;;
	alpha)
		U_value=$init_hubbard_u
		alpha_value=$i_u_value
		;;
	*)
		echo "invalid job_type input: $job_type, exit 1"
		exit 1
		;;
	esac
	# Replace current values in HUBBARD_U with $U_value
	sed -i -E "s/([spdf]):.*/\1: $U_value/g" "$cell_file"
	echo "Initiate U to $U_value"
	printf "\n" >>"$cell_file"
	cat "$cell_file" >"$cell_file".bak
	awk '/%BLOCK HUBBARD_U/,/%ENDBLOCK HUBBARD_U/' "$cell_file" | awk '{sub(/:.*/, u_value)gsub(/_U/, "_ALPHA")}1' u_value=": $alpha_value" >>"$cell_file".bak
	echo "Initiate Alpha to $alpha_value"
	mv "$cell_file".bak "$cell_file"
}

function cell_before {
	local cell_file=$1
	local init_hubbard_u=$2
	local i=$3
	local job_type=$4
	local u_value
	u_value=$(echo "$init_hubbard_u $i" | awk '{printf "%.14f0", $1+$2}')
	sed -i "s/\r//" "$cell_file"
	hubbard_before "$init_hubbard_u" "$i" "$job_type"
	echo -e "---------------------------------------------------------------------\nFor $cell_file:"
	awk '/%BLOCK HUBBARD_U/,/%ENDBLOCK HUBBARD_U/' "$cell_file"
	echo -e "\n"
	awk '/%BLOCK HUBBARD_ALPHA/,/%ENDBLOCK HUBBARD_ALPHA/' "$cell_file"
	echo "xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"
}

function param_before_perturb {
	local param_file=$1
	local init_elec_energy_tol=$2
	# remove \r from Windows generated files
	sed -i "s/\r//" "$param_file"
	sed -i '/^task.*/a !continuation : default' "$param_file"
	sed -i "s/\(elec_energy_tol :\).*/\1 $init_elec_energy_tol/" "$param_file"
	sed -i '/^fine_grid_scale.*/d' "$param_file"
	sed -i -E "s/(grid_scale)[ :]+[0-9.]+/\1 : 1.750000000000000/" "$param_file"
}

function setup_before_perturb {
	local init_hubbard_u=$1
	local i=$2
	local init_elec_energy_tol=$3
	local job_type=$4
	local folder_name=U_"$i"_"$job_type"
	# create new folder U_x
	echo "create $folder_name"
	mkdir -p "$folder_name"
	# copy files without '.castep' to U_x
	echo "copy files"
	find . -maxdepth 1 -type f -not -name "*.castep" -not -name "*.txt" -not -name "*.csv" -not -name "*.xsd" -not -name "*.xms" | xargs -I {} cp {} "$folder_name"/{}
	echo "$(pwd)"
	local cell_file
	cell_file=$(find ./"$folder_name" -maxdepth 1 -type f -name "*.cell")
	# setup cell
	cell_before "$cell_file" "$init_hubbard_u" "$i" "$job_type"
	# setup param
	local param_file
	param_file=$(find ./"$folder_name" -maxdepth 1 -type f -name "*.param")
	param_before_perturb "$param_file" "$init_elec_energy_tol"
	# return new folder_name
	setup_init_folder="$folder_name"
}

function param_after_perturb {
	local param_file=$1
	# remove "!" before continuation:default
	sed -i "s/^!//" "$param_file"
	# Divide elec_energy_tol by 10 to 1e-6
	local new_elec_energy_tol
	new_elec_energy_tol=$(echo "$init_elec_energy_tol" 10 | awk '{printf "%e", $1/$2}')
	sed -i -E "s/(elec_energy_tol :).*/\1 $new_elec_energy_tol/" "$param_file"
}

function hubbard_alpha_after_perturb {
	# cell file is the before perturb one
	local cell_file=$1
	local after_value=$2
	# read init alpha
	awk '/%BLOCK HUBBARD_ALPHA/,/%ENDBLOCK HUBBARD_ALPHA/ {sub(/: .*/, a)}1' a=": $after_value" "$cell_file" >"$cell_file".bak
}

function cell_after_perturb {
	local cell_file=$1
	local reference_cell=$2
	local perturb_step=$3
	local update_alpha_value=$4
	local ref_value
	ref_value=$(awk '/%BLOCK HUBBARD_ALPHA/,/%ENDBLOCK HUBBARD_ALPHA/' "$reference_cell" | awk 'NR==2 {print $4}')
	local after_value
	after_value=$(echo "$ref_value" "$update_alpha_value" | awk '{printf "%.14f0", $1+$2}')
	hubbard_alpha_after_perturb "$cell_file" "$after_value"
	echo "---------------------------------------------------------------------"
	echo -e "$cell_file\nPerturbation count: $perturb_step\nUpdate alpha to $after_value"
	awk '/%BLOCK HUBBARD_ALPHA/,/%ENDBLOCK HUBBARD_ALPHA/' "$cell_file.bak"
	echo "xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"
	mv "$cell_file".bak "$cell_file"
}

function setup_after_perturb {
	local perturb_step=$1
	local update_alpha_value=$2
	local folder_name=$3
	local new_folder_name="$folder_name""_$perturb_step"
	local dest="$folder_name/$new_folder_name"
	mkdir -p "$dest"
	find ./"$folder_name" -maxdepth 1 -type f -not -name "*.castep" -not -name "*.txt" -not -name "*.csv" -print0 | xargs -0 -I {} cp {} "$dest"
	local reference_cell
	reference_cell=$(find ./"$folder_name" -maxdepth 1 -type f -name "*.cell")
	local param_file
	param_file=$(find ./"$dest" -maxdepth 1 -type f -name "*.param")
	# setup param after perturbation
	param_after_perturb "$param_file"
	local cell_file
	cell_file=$(find ./"$dest" -maxdepth 1 -type f -name "*.cell")
	# setup cell after perturbation
	cell_after_perturb "$cell_file" "$reference_cell" "$perturb_step" "$update_alpha_value"
	# return new folder name
	setup_next_folder=$dest
}

function start_job {
	local current_dir
	current_dir=$(pwd)
	local job_dir=$1
	local job_type=$2
	local log_path=$3
	local result_path=$4
	local job_name
	job_name=$(find ./"$job_dir" -maxdepth 1 -type f -name "*.cell" | awk '{filename=$NF; sub(/\.[^.]+$/, "", filename); print filename}')
	local castep_command
	local castep_file="$job_name.castep"
	# Early exit 1 if the job has been done.
	if [[ -f "$castep_file" && "$(grep -c "Finalisation time" "$castep_file")" -gt 0 ]]; then
		echo "Current castep job has been completed! Skip now"
		write_data_converged "$castep_file" "$job_name" "$job_dir" "$job_type" "$result_path"
		return 1
	else
		cd "$job_dir" || exit 1
		case $job_type in
		U | u)
			cp "$castep_program_u_path" "$current_dir"/"$job_dir"/"$castep_program_u"
			castep_command="$castep_command_u"
			;;
		alpha | Alpha)
			cp "$castep_program_alpha_path" "$current_dir"/"$job_dir"/"$castep_program_alpha"
			castep_command="$castep_command_alpha"
			;;
		*) exit 1 ;;
		esac
		# Here is the command to start calculation
		# Use a single & to move the job to background
		# standalone when command needs jobname
		# $castep_command "$job_name" 2>&1 | tee -a "$current_dir"/log_"$job_type".txt
		# cluster, only script needed
		$castep_command 2>&1 | tee -a "$current_dir"/log_"$job_type".txt
		cd "$current_dir" || exit 1
		monitor_job_done "$job_dir" "$job_type" "$result_path"
	fi
}

function monitor_job_done {
	local dest=$1
	local job_type=$2
	local local_result_path=$3
	# get job name by extracting filestem
	# find under destination
	local jobname
	jobname=$(find ./"$dest" -maxdepth 1 -type f -name "*.cell" | awk '{filename=$NF; sub(/\.[^.]+$/, "", filename); print filename}')
	local castep_file="$jobname.castep"
	# Wait for generation of `.castep` file
	until [ -f "$castep_file" ]; do
		printf "Waiting for generation of %s\r" "$castep_file"
		sleep 1
	done
	echo -e "\nFound: $castep_file"
	# Monitor appearance of "Finalised time"
	local count
	count=$(grep -c "Finalisation time" "$castep_file")
	while [ "$count" -lt 1 ]; do
		printf "Waiting for job completion...\r"
		count=$(grep -c "Finalisation time" "$castep_file")
		sleep 1
	done
	echo -e "\nCalculation completed!"
	finished_castep_file="$castep_file"
	finished_job_name="$jobname"
	# setup after perturb
	write_data_converged "$finished_castep_file" "$finished_job_name" "$dest" "$job_type" "$local_result_path"
}

function format_data_output {
	local castep_file=$1
	local finished_job_name=$2
	local species_id=$3
	local spin=$4
	local is_converged=$5
	local write_path=$6
	local result
	# Remove the redundant space after comma, in `awk` command
	result=$(grep -Ei "[[:blank:]]+${species_id}[[:blank:]]+${spin} Total" "$castep_file" | awk 'NR==1 {printf "%.16f,", $4}; NR==2 {printf "%.16f,", $4}; END {printf "%.16f", $4} ORS=""')
	printf "%s,%i,%i,%s,%s\n" "$finished_job_name" "$species_id" "$spin" "$result" "$is_converged" >>"$write_path"
}

function write_data_converged {
	local castep_file=$1
	local finished_job_name=$2
	local dest=$3
	local cell_file
	cell_file=$(find ./"$dest" -maxdepth 1 -type f -name "*.cell")
	local job_type="$4"
	local local_result_path=$5
	local instant_result
	instant_result=result_"$job_type".csv
	local number_of_species
	number_of_species=$(awk '/%BLOCK HUBBARD_U/,/%ENDBLOCK HUBBARD_U/ {if (NF>2) print}' "$cell_file" | wc -l)
	for species_id in $(seq 1 "$number_of_species"); do
		for spin in 1 2; do
			format_data_output "$castep_file" "$finished_job_name" "$species_id" "$spin" "true" "$local_result_path"
			format_data_output "$castep_file" "$finished_job_name" "$species_id" "$spin" "true" "$instant_result"
		done
	done
}

function read_data {
	local i=$1
	local job_type=$2
	local folder_name=U_"$i"_"$job_type"
	local from_local_sheet=$3
	local to_total_sheet=$4
	cat "$folder_name"/"$from_local_sheet" >>"$to_total_sheet"
}

# after cd into SEED_PATH
function routine {
	local init_hubbard_u=$1
	local current_input_U=$2
	local init_elec_energy_tol=$3
	local job_type=$4
	local log_path=$5
	local perturb_init_alpha=$6
	local perturb_increment=$7
	local perturb_final_alpha=$8
	setup_before_perturb "$init_hubbard_u" "$current_input_U" "$init_elec_energy_tol" "$job_type"
	init_folder="$setup_init_folder"
	local local_result_path="$init_folder"/result_"$job_type".csv
	: >"$local_result_path"
	# run castep
	# castep $SEED_PATH/$SEED_NAME
	# monitor result
	start_job "$init_folder" "$job_type" "$log_path" "$local_result_path"
	# echo  "Setup next perturbation step\r"
	local perturb_alpha_values
	perturb_alpha_values=$(seq "$perturb_init_alpha" "$perturb_increment" "$perturb_final_alpha" | awk '{printf "%.14f0 ", $1}')
	local step
	step=0
	for alpha_add in $perturb_alpha_values; do
		step=$((step + 1))
		setup_after_perturb "$step" "$alpha_add" "$init_folder"
		next_folder=$setup_next_folder
		start_job "$next_folder" "$job_type" "$log_path" "$local_result_path"
	done
}

function create_log {
	local job_type=$1
	# local current_dir
	# current_dir=$(pwd)
	local log_path
	log_path="$SEED_PATH"/log_"$job_type".txt
	true >"$log_path"
	echo log_path
}

function serial {
	cd "$SEED_PATH" || exit 1
	echo "$(pwd)"
	for curr_u in $(seq "$init_input_u" "$u_step" "$final_U"); do
		routine "$init_hubbard_u" "$curr_u" "$init_elec_energy_tol" "$job_type" "$log_path" "$PERTURB_INIT_ALPHA" "$PERTURB_INCREMENT" "$PERTURB_FINAL_ALPHA"
	done
	for curr_u in $(seq "$init_input_u" "$u_step" "$final_U"); do
		read_data "$curr_u" "$job_type" result_"$job_type".csv result_"$job_type"_final.csv
	done
	echo "Result:"
	cat result_"$job_type"_final.csv
}

function parallel {
	local N=$1
	cd "$SEED_PATH" || exit 1
	for curr_u in $(seq "$init_input_u" "$u_step" "$final_U"); do
		(
			# .. do your stuff here
			routine "$init_hubbard_u" "$curr_u" "$init_elec_energy_tol" "$job_type" "$log_path" "$PERTURB_INIT_ALPHA" "$PERTURB_INCREMENT" "$PERTURB_FINAL_ALPHA"
		) &

		# allow to execute up to $N jobs in parallel
		if [[ $(jobs -r -p | wc -l) -ge $N ]]; then
			# now there are $N jobs already running, so wait here for any job
			# to be finished so there is a place to start next one.
			wait -n
		fi
	done

	# no more jobs to be started but wait for pending jobs
	# (all need to be finished)
	wait
	for curr_u in $(seq "$init_input_u" "$u_step" "$final_U"); do
		read_data "$curr_u" "$job_type" result_"$job_type".csv result_"$job_type"_final.csv
	done
	echo "Result:"
	cat result_"$job_type"_final.csv
	echo "all done"
}

function grep_data_with_check {
	local write_path=$1
	local castep_file
	castep_file=$(find . -maxdepth 1 -type f -name "*.castep")
	local job_step
	job_step=$(pwd | awk -F '/' '{print $NF}')
	local job_name
	job_name="$job_step"/$(echo "$castep_file" | awk -F '/' '{filename=$NF; sub(/\.[^.]+$/, "", filename); print filename}')
	local cell_file
	cell_file=$(find ./"$dest" -maxdepth 1 -type f -name "*.cell")
	local number_of_species
	number_of_species=$(awk '/%BLOCK HUBBARD_U/,/%ENDBLOCK HUBBARD_U/ {if (NF>2) print}' "$cell_file" | wc -l)
	local is_converged
	if [[ $(grep -c "Finalisation time" "$castep_file") -eq 0 ]]; then
		is_converged="false"
	else
		is_converged="true"
	fi

	for species_id in $(seq 1 "$number_of_species"); do
		for spin in 1 2; do
			format_data_output "$castep_file" "$job_name" "$species_id" "$spin" "$is_converged" "$write_path"
		done
	done
}

function after_read_in_every_U {
	local cwd
	cwd=$(pwd)
	local result_path
	result_path=$(pwd)/local_result_"$job_type"_post.csv
	: >"$result_path"
	grep_data_with_check "$result_path"
	for step in $(seq 1 "$PERTURB_TIMES"); do
		local target_dir
		target_dir=U_"$u"_"$job_type"_"$step"
		cd "$target_dir" || {
			echo "Directory: $target_dir does not exist, skip"
			continue
		}
		grep_data_with_check "$result_path"
		cd "$cwd" || exit 1
	done
}

function after_read {
	cd "$SEED_PATH" || {
		echo "$SEED_PATH does not exist"
		exit 1
	}
	local current_dir
	current_dir=$(pwd)
	local post_total_path
	post_total_path=result_"$job_type"_post_read.csv
	printf "Jobname,Channel ID,Spin,Before SCF,1st SCF,Last SCF,Converged\n" >"$post_total_path"
	for u in $(seq "$init_input_u" "$u_step" "$final_U"); do
		local target_dir
		target_dir=U_"$u"_"$job_type"
		echo "$current_dir"
		cd "$target_dir" || {
			echo "Directory: $target_dir does not exist, skip"
			continue
		}
		after_read_in_every_U
		cd "$current_dir" || exit 1
		read_data "$u" "$job_type" local_result_"$job_type"_post.csv "$post_total_path"
	done
	cat "$post_total_path"
}

function use_hubbard_data {
	local current_dir
	local u_source
	local alpha_source
	current_dir=$(pwd)
	cd $DATA_SOURCE || exit 1
	# Under $current_dir/$DATA_SOURCE
	read -r -e -p "Please input the directory of Hubbard U task (e.g.: ZnO_u_0_2_12_0.05_0.05_0.25_STEPS_5): " u_source
	while [ u_source == '' ]; do
		read -r -e -p "Please input the directory of Hubbard U task (e.g.: ZnO_u_0_2_12_0.05_0.05_0.25_STEPS_5): " u_source
	done
	read -r -e -p "Please input the directory of Hubbard Alpha task (e.g.: ZnO_alpha_0_2_12_0.05_0.05_0.25_STEPS_5): " alpha_source
	while [ alpha_source == '' ]; do
		read -r -e -p "Please input the directory of Hubbard Alpha task (e.g.: ZnO_alpha_0_2_12_0.05_0.05_0.25_STEPS_5): " alpha_source
	done
	local u_U_steps
	local alpha_U_steps
	# Match U steps of Hubbard U task from folder name
	u_U_steps=$(echo "$u_source" | sed -r 's/.*_u_([0-9]+_[0-9]+_[0-9]+).*/\1/')
	# Match U steps of Hubbard Alpha task from folder name
	alpha_U_steps=$(echo "$alpha_source" | sed -r 's/.*_alpha_([0-9]+_[0-9]+_[0-9]+).*/\1/')
	# Guard: if $u_U_steps and $alpha_U_steps are not the same, exit 1 the program
	if [ "$u_U_steps" != "$alpha_U_steps" ]; then
		echo "$u_source and $alpha_source do not match; they have different settings of starting U, U step and ending U: $u_U_steps vs $alpha_U_steps"
		exit 1
	fi

	local u_perturb_value
	local alpha_perturb_value
	u_perturb_value=$(echo "$u_source" | sed -E 's/.*_[0-9.]+_([0-9.]+)_[0-9.]+_STEPS.*/\1/')
	alpha_perturb_value=$(echo "$alpha_source" | sed -E 's/.*_[0-9.]+_([0-9.]+)_[0-9.]+_STEPS.*/\1/')

	if [ "$u_perturb_value" != "$alpha_perturb_value" ]; then
		echo "Perturbation value of U ($u_perturb_value) and Alpha ($alpha_perturb_value) do not match; \
this is currently unsupported by the plotting program 'hubbard_data'."
		exit 1
	fi

	PERTURB_INCREMENT=$u_perturb_value

	local plot_dir
	plot_dir=plot_"$u_U_steps"
	if [ ! -d $plot_dir ]; then
		mkdir $plot_dir
	fi
	if [[ -f "$u_source"/result_u_final.csv && -f "$alpha_source"/result_alpha_final.csv ]]; then
		# Make sure the csv does not have extra space between commas
		sed -i 's/, /,/g' "$u_source"/result_u_final.csv
		sed -i 's/, /,/g' "$alpha_source"/result_alpha_final.csv
		cp "$u_source"/result_u_final.csv "$alpha_source"/result_alpha_final.csv $plot_dir || exit 1
		# back to $current_dir, which is supposed to have `hubbard_data` bin at the current directory.
		cd $current_dir || exit 1
		./hubbard_data -s "$DATA_SOURCE"/$plot_dir "$PERTURB_INCREMENT"
	elif [ ! -f "$u_source"/result_u_final.csv ]; then
		echo "$u_source does not have 'result_u_final.csv'! Please double check the files."
		echo "Now exiting..."
		exit 1
	else
		echo "$alpha_source does not have 'result_alpha_final.csv'! Please double check the files."
		echo "Now exiting..."
		exit 1
	fi
}
