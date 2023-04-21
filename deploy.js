const { execSync } = require("child_process");
const os = require("os");

const setOutput = (name, value) => {
  process.stdout.write(name + " " + value + " " + os.EOL);
};

const setFailed = (message) => {
  process.exitCode = ExitCode.Failure;

  process.stdout.write(os.EOL);
};

const createCatFile = (email, api_key) => `cat >~/.netrc <<EOF
machine api.heroku.com
    login ${email}
    password ${api_key}
machine git.heroku.com
    login ${email}
    password ${api_key}
EOF`;

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

  //
  execSync(createCatFile(env.heroku_email, env.heroku_api_secret));
  console.log("Created and wrote to ~/.netrc");

  // login
  execSync("heroku container:login");
  console.log("Successfully logged into heroku");

  addRemote(env.heroku_app_name);
  execSync(
    `heroku config:set --app=${app_name} APP_ENVIRONMENT=${env.app_environment}`
  );

  try {
    execSync("git push heroku main");
    setOutput("status", "Successfully deployed heroku app from branch main");
  } catch (e) {
    setFailed(err.toString());
  }
})();
