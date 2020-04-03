let completions =
  {|#compdef nodo

for l in $(nodo completion -- $words); do
    compadd $l
done
|}
