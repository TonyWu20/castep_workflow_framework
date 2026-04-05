#! /usr/bin/sh
touch "$1.castep"
{
	echo "           1           1 Total:    4.88454712510949       Mz:"
	echo "           1           2 Total:    2.04480063601341       Mz:"
	echo "           1           1 Total:    4.88222767868911       Mz:"
	echo "           1           2 Total:    2.03625090967629       Mz:"
	echo "           1           1 Total:    4.88417863385162       Mz:"
	echo "           1           2 Total:    2.03022846329140       Mz:"
} >>"$1.castep"
echo "Finalisation time" >>"$1.castep"
