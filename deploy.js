const { execSync } = require("child_process");

const createCatFile = (email, api_key, reset = true) => {
  if (reset) {
    execSync("rm -rf ~/.netrc");
  }

  return `cat >~/.netrc <<EOF
machine api.heroku.com
    login ${email}
    password ${api_key}
machine git.heroku.com
    login ${email}
    password ${api_key}
EOF`;
};

const getEnv = (name, options) => {
  const val = process.env[`${name.replace(/ /g, "_").toUpperCase()}`] || "";
  if (options && options.required && !val) {
    throw new Error(`Input required and not supplied: ${name}`);
  }

  if (options && options.trimWhitespace === false) {
    return val;
  }

  return val.trim();
};

const addRemote = (app_name) => {
  execSync("heroku git:remote --app " + app_name);
  console.log("Added git remote heroku");
};

let env = {
  heroku_api_secret: getEnv("HEROKU_API_SECRET"),
  heroku_app_name: getEnv("HEROKU_APP_NAME"),
  heroku_email: getEnv("HEROKU_EMAIL"),
  app_environment: getEnv("APP_ENVIRONMENT"),
  reset_netrc: getEnv("RESET_NETRC") === "true",
};

(async () => {
  execSync(`git config user.name "Heroku-Deploy"`);
  execSync(`git config user.email "${env.email}"`);
  const status = execSync("git status --porcelain").toString().trim();
  if (status) {
    execSync(
      'git add -A && git commit -m "Commited changes from previous actions"'
    );
  }

  const isShallow = execSync(
    "git rev-parse --is-shallow-repository"
  ).toString();

  // If the Repo clone is shallow, make it unshallow
  if (isShallow === "true\n") {
    execSync("git fetch --prune --unshallow");
  }

  execSync(
    createCatFile(env.heroku_email, env.heroku_api_secret, env.reset_netrc)
  );
  console.log("Created and wrote to ~/.netrc");

  // login
  execSync("heroku container:login");
  console.log("Successfully logged into heroku");

  addRemote(env.heroku_app_name);

  execSync(
    `heroku config:set --app=${env.heroku_app_name} APP_ENVIRONMENT=${env.app_environment}`
  );

  try {
    execSync(`git push heroku main:refs/heads/main --force`, {
      stdio: ["pipe", process.stdout, process.stderr],
    });
    process.exit(0);
  } catch (e) {
    console.error("Error while pushing: ", e);
    process.exit(1);
  }
})();
