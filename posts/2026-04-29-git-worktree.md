---
title: Pages workflow
date: 2026-04-29
template: ./templates/post.html
---
A quick look at how I publish this website.

# Hosting and Publishing

This website (at least at the time of writing) is hosted on GitHub pages[^1].

[^1]: There are lots of issues with GitHub that I'm well aware of, not least its awful uptime.  I still use it because it offers a lot of free computer time, in particular with GitHub.dev, which I use as I'm not convinced my old laptop would survive being used as a development server while I'm procrastinating at work by writing here!  Also the free web hosting is nice, I don't feel like putting £10/month on a vps & domain name.  Would be nice at some point though, could maybe even experiment with having dynamic backend code.

In order to publish on GitHub pages, there are several main methods.
Use actions to build your website and upload the build output files as your website.
Use the root of your repository as the website and host the file system directly.  Optionally builds the repository using the Jekyll static site builder (unless a `.nojekyll` file is included)
Same as 2, but uses the `/docs` folder.
Same as 2, but uses the root of the `gh-pages` branch.

Personally, I don't like option 1.  It allows building the site however you want, however requires you to get the build system to work on GitHub actions, with all the weird yaml files that requires.  You also cannot see what the actual code being hosted is.

The other methods all have the option of using Jekyll to build the site.  Personally that isn't useful, as I'd like to write the site generator myself to make it easier to control exactly what is output.

That leaves 3 options which all require the exact files which will be hosted to be within the repository.  With option 2, it doesn't really work for having your own build system files be within the repository as it will host your source code instead as well as your website.

Option 3 is the one I previously used.  The /docs folder contained the output of the build system.  This has the issue that you will have your build artifacts committed to the repository and you have to remember to not add all updated artifacts if you want to add to the code only.  It also makes your git history messy, with constant changes to the artifacts, e.g. a minor change to a template file becomes a change on all built files.  It has the advantage however, that it is very easy to setup a build system to output to a folder within your repository.

Option 4 is the one I now use.  I initially didn't use it as it seemed hard to work out how to build the website and push the build files to a different branch.  Now I've worked out how, the messy history of the build artifacts can be in one branch and the code is in another branch.

# Git Workflow

So, here is how I have managed to make option 4 work:

## Step 0:
Ensure everything is committed already incase anything gets removed or done wrong.

## Step 1 - Create a new branch for the build files:
```sh
git checkout --orphan gh-pages
git rm -rf .
git commit --allow-empty -m "Initial deploy"
git push --set-upstream origin gh-pages
```
This creates the branch and pushes it.  You could add initial build files instead of an empty commit.  Note that this removes all files from the repository!

If the branch already exists, you don't have to do this step.

## Step 2 - Setup your repository:
```sh
git checkout main
```
You will then have to choose the name of your build directory.  I have chosen `/build` in here, however you can pick whatever you want.

Add `/build` to your `.gitignore` file (make one if you don't already have one).  Git metadata and your build files will be created within the folder that shouldn't be committed to the main branch.

```sh
git worktree add build gh-pages
```
This creates a view of the gh-pages branch you created in step 2 within the `/build` directory.

## Step 3 - Build and commit:
Run your code to put the website's built files within the `/build` directory.  Note that the folder will contain a `.git` file, which should NOT be removed by your build scripts! This means you cannot just delete the build directory to remove old artifacts.

Then, to commit your built files:
```sh
cd build
git add .
git commit -m "Deploy"
git push
```
In my case, the git UI within vscode can display the build directory as a worktree with a separate commit button, without having to run the above commands manually.

## Step 4 - Done
That's it!  Use `git commit` from the root of the repository to update the code and from the build directory to deploy.
