{
  homebrew = {
    casks = [
      { name = "firefox"; greedy = true; }
      { name = "google-chrome"; greedy = true; }
      { name = "arc"; }
      { name = "1password"; greedy = true; }
      { name = "raycast"; greedy = true; }
      { name = "karabiner-elements"; greedy = true; }
    ];
  };

  programs.zsh = {
    enable = true;
    history = {
      size = 100000;
      save = 100000;
      ignoreDups = true; # prevent duplicates
      share = true;
      extended = true;
    };
  };

  # Package list
  environment.systemPackages = with pkgs;
  [
    vim
    git
    ripgrep # fast search
    fd # find alternative
  ];
}
