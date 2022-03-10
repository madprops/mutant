<?php
  $recaptcha_key = trim(file_get_contents("recaptcha.key"));
  $ch = curl_init();
  curl_setopt($ch, CURLOPT_URL, "https://www.google.com/recaptcha/api/siteverify?secret=".$recaptcha_key."&response=".$_POST["g-recaptcha-response"]);
  curl_setopt($ch, CURLOPT_RETURNTRANSFER, true);
  $response = curl_exec($ch);
  curl_close($ch);
  $response = json_decode($response);

  if($response->success) {
    // OK
  } else {
    echo "reCAPTHCA verification failed, please try again.";
    exit(0);
  }

  error_reporting(E_ALL);
  ini_set("display_errors", "On");
  $uploaded = [];
  $paths = [];

  function randnumb($length = 3) {
    return substr(str_shuffle(str_repeat($x="0123456789", ceil($length/strlen($x)) )),1,$length);
  }

  $ok = true;
  $num_images = 1;
  $images = [];

  for ($i = 1; $i <= 4; $i++) {
    $error = $_FILES["image_" . $i]['error'];
    if($error == UPLOAD_ERR_OK) {
      array_push($images, $_FILES["image_" . $i]);
    }
  }

  if(count($images) == 0) {
    echo "No image was provided.";
    $ok = false;
  }

  if(count($images) > 4) {
    echo "Too many images were provided.";
    $ok = false;
  }

  for ($i = 0; $i < count($images); $i++) {
    if($images[$i]["size"] > 9999999) {
      echo "File is too large.";
      $ok = false;
    }
  }

  if($ok) {
    $fstring = "";
    $start = true;

    for ($i = 0; $i < count($images); $i++) {
      $fname = basename($images[$i]["name"]);
      $ext = substr($fname, strrpos($fname, '.') + 1);
      $fname2 = time() . "_" . randnumb() . "_" . $i . "." . $ext;
      $target = "uploads/" . $fname2;
      global $uploaded;

      if(move_uploaded_file($images[$i]["tmp_name"], $target)) {
        array_push($uploaded, $target);
        $fstring = trim($fstring . " " . $target);
      } else {
        echo "There was an error uploading the image(s).";
        $start = false;
        break;
      }
    }

    if($start) {
      process($fstring);
    }
  }

  function process($filestring) {
    $modes = [];
    $num_effects = 8;
    $delay = 0.25;

    for ($i = 1; $i <= $num_effects; $i++) {
      if(isset($_POST["effect_" . $i])) {
        array_push($modes, $i);
      }
    }

    if(count($modes) == 0) {
      echo "No effects selected.";
      exit(0);
    }

    $modestring = implode(",", $modes);

    if(isset($_POST["delay"])) {
      $delay = $_POST["delay"];
    }

    $date1 = microtime(true);

    try {
      $cmd = "rust/target/release/mutant " . $modestring . " " . $delay . " " . $filestring;
    } catch (Exception $e) {
      cleanup();
      exit(0);
    }

    global $paths;
    $paths = explode(" ", exec($cmd));
    $date2 = microtime(true);

    $style = "body, html {
      background-color: black;
      color: white;
      font-family: sans-serif;
      font-size: 18px;
    } 
      
    .item {
      display: inline-block;
      padding: 20px;
    } 
      
    .title {
      padding-bottom: 5px;
      display: block;
    }
      
    .info {
      margin-left: 20px;
      margin-right: 20px;
      margin-top: 20px;
      margin-bottom: 15px;
      padding: 5px;
      background-color: #32343d
    }";

    $topheader = "<title>Mutant Result</title>
    <meta http-equiv='Content-Type' content='text/html; charset=utf-8'>
    <link rel='shortcut icon' href='guy.png?v=1' type='image/x-icon'>";

    $diff = round(($date2 - $date1), 2);
    $seconds = "seconds";

    if($diff == 1) {
      $seconds = "second";
    }

    echo "<head>" . $topheader . "<style>" . $style . "</style></head>";

    $effects = "effects";

    if(count($modes) == 1) {
      $effects = "effect";
    }

    $imgs = "The images";

    if(count($modes) == 1) {
      $imgs = "The image";
    }

    echo "<div class='info'>" . count($modes) . " " . $effects . " processed in " . $diff . " " . $seconds . "</div>";

    $effect_name_1 = "Glitch";
    $effect_name_2 = "Wave";
    $effect_name_3 = "Mirror";
    $effect_name_4 = "Static";
    $effect_name_5 = "Glow";
    $effect_name_6 = "Glass";
    $effect_name_7 = "Color";
    $effect_name_8 = "Chalk";

    try {
      for ($i = 0; $i < count($modes); $i++) {
        echo "<div class='item'><div class='title'>" . ${"effect_name_" . $modes[$i]} . " | Size: " . round(filesize($paths[$i]) / 1024 / 1024, 1) . " MiB</div>";
        $data = file_get_contents($paths[$i]);
        echo "<img class='image' src='data:image/gif;base64," . base64_encode($data) . "'></div>";
      }
    } catch (Exception $e) {
      cleanup();
      exit(0);
    }

    cleanup();
  }

  function cleanup() {
    global $uploaded;
    global $paths;

    for ($i = 0; $i < count($paths); $i++) {
      unlink($paths[$i]);
    }

    for ($i = 0; $i < count($uploaded); $i++) {
      unlink($uploaded[$i]);
    }
  }
?>