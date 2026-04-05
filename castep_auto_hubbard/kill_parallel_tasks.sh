ps -ef | grep "auto_hubbard" | awk '{print $2}' | xargs -I {} kill {}
