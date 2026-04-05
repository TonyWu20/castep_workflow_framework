#! /bin/bash
# Automatic hubbard_U increment calculation workflow
# !!!! Caution: substitute function `faux_castep_run` by "$castep_command"
#
# Please read comments in this script if you want to understand and/or
# modify the necessary parts for actual needs.

# 1. Setup before
init_hubbard_u=0.000000010000000
init_elec_energy_tol=1e-5
# !!! Please adjust this variable to the actual command to
# start castep calculation.
castep_program_u="faux_castep_run.sh"
castep_program_alpha="faux_castep_run.sh"
castep_command_u="bash ./${castep_program_u} GDY_111_Fe_U"
castep_command_alpha="bash ./${castep_program_alpha} GDY_111_Fe_U"
script_path=$(pwd)

source "$(dirname "$0")"/functions_linux.sh

# List of running modes for selection
RUN_MODES=(serial parallel read draw)
# First  argument is running mode
if [[ $1 == '' ]]; then
	PS3="Please choose running mode (enter number): "
	select choice in "${RUN_MODES[@]}"; do
		case $choice in
		serial | parallel | read | draw)
			run_mode="$choice"
			break
			;;
		*) echo "Invalid option $choice" ;;
		esac
	done
elif [[ $1 == 'serial' || $1 == 'parallel' || $1 == 'read' || $1 == 'draw' ]]; then
	run_mode=$1
else
	echo "Invalid input of running mode (serial/parallel/read/draw), please restart the program"
	exit
fi

# init_input_U=0
# U_increment=2
# final_U=12
# set init input U, U step, final U
case $run_mode in
draw)
	if [[ $2 == '' ]]; then
		read -r -e -p "Please enter the seed directory, which has the folders starting with 'SEED_u_x_x_x...', 'SEED_alpha_x_x_x': " data_source
		DATA_SOURCE=$data_source
	else
		DATA_SOURCE=$2
	fi
	use_hubbard_data
	;;
serial | parallel)
	# Second argument is seed file folder path
	if [[ $2 == '' ]]; then
		read -r -e -p "seed file folder path:" SEED_PATH
		# Force to give input
		while [[ $SEED_PATH == '' ]]; do
			read -r -e -p "seed file folder path:" SEED_PATH
		done
	else
		SEED_PATH=$2
	fi

	# Third argument is job type: u or alpha
	# Will be reconfirmed for blank or invalid input
	job_type_arg=$3
	job_type_input "$job_type_arg"
	if [[ $4 == '' ]]; then
		read -r -e -p "Initial input U (default to 0): " init_input_u
		init_input_U=${init_input_u:-0}
	elif [[ $4 =~ ^[0-9]+$ ]]; then
		init_input_U=$4
	else
		echo "Input init U is not a valid integer; run by default 0"
	fi
	if [[ $5 == '' ]]; then
		read -r -e -p "Input U increment step (default to 2): " step_u
		U_increment=${step_u:-2}
	elif [[ $5 =~ ^[0-9]+$ ]]; then
		U_increment=$5
	else
		echo "Input U increment step is not a valid integer; run by default 2"
	fi
	if [[ $6 == '' ]]; then
		read -r -e -p "Final input U (default to 12): " final_u
		final_U=${final_u:-12}
	elif [[ $6 =~ ^[0-9]+$ ]]; then
		final_U=$6
	else
		echo "Input final U is not a valid integer; run by default 12"
	fi

	#  the initial alpha value shift in perturbation
	if [[ $7 == '' ]]; then
		read -r -e -p "Perturb initial Δalpha (default to 0.05): " init_alpha
		PERTURB_INIT_ALPHA=${init_alpha:-0.05}
	elif [[ $7 =~ ^[+-]?[0-9]+\.?[0-9]*$ ]]; then
		PERTURB_INIT_ALPHA=$7
	else
		echo "Input init Δalpha is not a valid float number; run by default 0.05"
	fi

	# Fifth argument is the increment of Hubbard_alpha
	if [[ $8 == '' ]]; then
		read -r -e -p "Perturb increment (e.g. +0.05/per step): " increment
		PERTURB_INCREMENT=${increment:-0.05}
		echo "Increment of alpha: $PERTURB_INCREMENT"
	else
		PERTURB_INCREMENT=$8
	fi

	# Fifth argument is the initial U
	if [[ $9 == '' ]]; then
		read -r -e -p "Perturb final Δalpha (default to 0.25): " final_alpha
		PERTURB_FINAL_ALPHA=${final_alpha:-0.25}
	elif [[ $9 =~ ^[+-]?[0-9]+\.?[0-9]*$ ]]; then
		PERTURB_FINAL_ALPHA=$9
	else
		echo "Input final Δalpha is not a valid float number; run by default 0.25"
	fi

	PERTURB_TIMES=$(seq "$PERTURB_INIT_ALPHA" "$PERTURB_INCREMENT" "$PERTURB_FINAL_ALPHA" | wc -l)
	echo "Init Δalpha=$PERTURB_INIT_ALPHA; increment=$PERTURB_INCREMENT; final Δalpha=$PERTURB_FINAL_ALPHA"
	echo -e "Perturbation times: $PERTURB_TIMES\n"
	setup "$init_hubbard_u" "$init_elec_energy_tol" "$init_input_U" "$U_increment" "$final_U" "$job_type"
	setup_new_seed_folder
	setup_perturbation "$PERTURB_INIT_ALPHA" "$PERTURB_INCREMENT" "$PERTURB_FINAL_ALPHA"
	setup_castep_command "$castep_command_u" "$castep_command_alpha" "$castep_program_u" "$castep_program_alpha"

	N=32
	case $run_mode in
	serial) serial ;;
	parallel) parallel $N ;;
	*) exit ;;
	esac
	;;
read)
	if [[ $2 == '' ]]; then
		read -r -e -p "seed file folder path:" SEED_PATH
	else
		SEED_PATH=$2
	fi

	# Second argument is job type: u or alpha
	# Will be reconfirmed for blank or invalid input
	job_type_arg=$3
	job_type_input "$job_type_arg"
	if [[ $4 == '' ]]; then
		read -r -e -p "Initial input U (default to 0): " init_input_u
		init_input_U=${init_input_u:-0}
	elif [[ $4 =~ ^[0-9]+$ ]]; then
		init_input_U=$4
	else
		echo "Input init U is not a valid integer; run by default 0"
	fi
	if [[ $5 == '' ]]; then
		read -r -e -p "Input U increment step (default to 2): " step_u
		U_increment=${step_u:-2}
	elif [[ $5 =~ ^[0-9]+$ ]]; then
		U_increment=$5
	else
		echo "Input U increment step is not a valid integer; run by default 2"
	fi
	if [[ $6 == '' ]]; then
		read -r -e -p "Final input U (default to 12): " final_u
		final_U=${final_u:-12}
	elif [[ $6 =~ ^[0-9]+$ ]]; then
		final_U=$6
	else
		echo "Input final U is not a valid integer; run by default 12"
	fi

	if [[ $7 == '' ]]; then
		read -r -e -p "How many times of perturbation did you set? " PERTURB_TIMES
	else
		PERTURB_TIMES=$7
	fi
	setup "$init_hubbard_u" "$init_elec_energy_tol" "$init_input_U" "$U_increment" "$final_U" "$job_type"
	after_read
	;;
*) exit ;;
esac
