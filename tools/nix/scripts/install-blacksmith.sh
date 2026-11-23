runHook preInstall
install -Dm755 $src $out/bin/blacksmith
runHook postInstall
