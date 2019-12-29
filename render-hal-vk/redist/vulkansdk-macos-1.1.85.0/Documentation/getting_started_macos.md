<!-- markdownlint-disable MD041 -->
[![LunarG][1]][2]

# Getting Started with the Vulkan SDK

## Version for macOS / iOS

Copyright &copy; 2015-2018 LunarG, Inc.

[![Creative Commons][3]][4]

[1]: https://vulkan.lunarg.com/img/LunarGLogo.png "www.LunarG.com"
[2]: https://www.LunarG.com/
[3]: https://i.creativecommons.org/l/by-nd/4.0/88x31.png "Creative Commons License"
[4]: https://creativecommons.org/licenses/by-nd/4.0/

[lxchangelink]: https://Vulkan.LunarG.com "LunarXchange"
[khronosvklink]: https://khronos.org/registry/vulkan "Khronos Vulkan website"
[tutoriallink]: https://vulkan.lunarg.com/doc/sdk/latest/linux/tutorial/html/index.html
[mvkuserguide]: https://github.com/KhronosGroup/MoltenVK/blob/master/Docs

**Note: This SDK will evolve over time to include additional content.
As such, the organization of the files within the SDK may change.
Please monitor the release notes for notifications of the movement of
any of the components.**

### About This Guide

This guide describes the requirements and procedure for installing
the LunarG Vulkan SDK for macOS.
This SDK provides the development and runtime components required to
build, run, and debug MoltenVK-subset Vulkan applications.
Refer to the LunarG Vulkan SDK, Documentation, and Known Issues at the
[LunarXchange website][lxchangelink] for the most current SDK information.

### Vulkan

The Vulkan API is a low overhead, explicit, cross-platform graphics API that provides
applications with direct control over the GPU, maximizing application performance.
For more information on the Vulkan specification and API,
refer to [Khronos.org][khronosvklink].
For tutorial-level information, refer to LunarG's Vulkan tutorial, which can be found
on the [LunarXchange website][tutoriallink].

### MoltenVK

This SDK provides partial Vulkan support though the use of the MoltenVK library
which is a "translation" or "porting" library that maps most of the Vulkan
functionality to the underlying graphics support on macOS and iOS platforms.
The MoltenVK library takes on the role of the Installable Client Driver (ICD)
from the point of view of the application and the Vulkan loader.
It is NOT a fully-conforming Vulkan driver for macOS or iOS devices.
Please see the [MoltenVK Runtime User Guide][mvkuserguide] on the MoltenVK GitHub
for more information.

#### iOS Support Note

This SDK includes MoltenVK libraries for iOS that allow targeting MoltenVK
applications for iOS from a macOS/Xcode development environment.
This support is in the form of a MoltenVK framework and a MoltenVK library for iOS
which can be used for building applications intended for deployment on iOS devices.

This SDK does not yet include Vulkan loader or validation layer libraries that are
ready for deployment on iOS systems.
Until these components become available for deployment on iOS, MoltenVK iOS applications
should link to the MoltenVK libraries directly without using the Vulkan loader.
Validation and other layer support is consequently not available on iOS without the Vulkan
loader.
Instructions for using the MoltenVK libraries directly are available in the
[MoltenVK Runtime User Guide][mvkuserguide].

### In Case of Trouble

If you have a problem with the SDK, please determine if it is a specific functional
problem with one of the components in the SDK.
Use the "SDK Origins" section in this document to find a list of repositories that
make up this SDK.
If you can determine that the problem is in a component that comes from one of these
repositories, please report the problem there.
For example, a problem with the Vulkan loader is best reported in the
`KhronosGroup/Vulkan-Loader` GitHub repository.

Otherwise, if you have a question with the SDK itself, such as how to use
the SDK components on macOS, please use the issue reporting system on
[LunarXchange][lxchangelink].

## Terminology

| Term | Description |
| ---- | ----------- |
| ICD | Installable Client Driver: A Vulkan-compatible display driver. In this SDK, the ICD is the MoltenVK library.|
| GLSL | OpenGL Shading Language|
| Layer | A library designed to work as a plug-in for the loader. It usually serves to provide validation and debugging functionality to applications. |
| Loader | A library which implements the Vulkan API entry points and manages layers, extensions, and drivers. It can be found in the SDK, as well as independent hardware vendor driver installs |
| MoltenVK | A library that maps most of the Vulkan functionality to underlying graphics support on macOS and iOS. |
| SPIR-V | Standard Portable Intermediate Representation: A cross-API intermediate language that natively represents parallel compute and graphics programs. |
| Vulkan | A low overhead, explicit graphics API developed by the Khronos Group and member companies. |
| WSI | Window System Integration  |
| Xcode | The preferred application development environment for macOS and iOS applications. |

## System Requirements

* MoltenVK can be run on iOS 9 and macOS 10.11 "El Capitan" devices
* MoltenVK references advanced OS frameworks during build
* All components in the SDK are built on macOS 10.12 "Sierra"
* Xcode 9 or greater is required for building apps using this SDK

## SDK Installation

### SDK Versioning

The components in this SDK are built with a specific version of
Khronos Vulkan API header, whose version is reflected in the SDK's version number.
For example, SDK version 1.0.xx.0 indicates that the SDK uses Vulkan header
revision 1.0.xx where:

* "1" is the Vulkan major version
* "0" is the Vulkan minor version
* "xx" is the Vulkan patch version (e.g., 24)

The last number in the SDK version indicates the revision of an SDK for the given
Vulkan header revision.
It is used in case it is necessary to release multiple SDKs for the same
version of Vulkan.

### Download the SDK

Download the LunarG Vulkan SDK from the [LunarXchange website][lxchangelink].
The SDK download file is named with the pattern:

    vulkansdk-macos-1.1.xx.0.tar.gz

Make note of the directory to which the file was downloaded.

#### Downloading With Safari Note

The Safari web browser is often configured to "open" files after downloading.
This may cause Safari to uncompress the `tar.gz` file after downloading,
resulting in a `tar` file.
In this case, you need to locate the `tar` file and not the `tar.gz` file
and proceed with the rest of the steps in the same manner with the `tar` file.

### Install the SDK

Installing is a simple operation involving expanding a "tar" archive file
into your work area.
The macOS SDK is intended to be installed anywhere the user can place files
such as the user's `$HOME` directory.
The SDK isn't configured to install to protected system locations such as
`/usr/lib` or `/System`.

If using `Finder`, navigate to the downloaded file and drag it to where you
want to install it.
Once the `tar.gz` or `tar` file is in the desired location, double-click
on it in `Finder` to expand the archive.
The archive expansion creates a directory containing the SDK contents that has
the same name as the file.

If using the command line in `Terminal`, use shell commands to move the `tar.gz`
file to where you want to install it and run `tar -xzf filename` to expand.
If the file is a `tar` file, then `tar -xf filename` will suffice.

Note that since each SDK version has a unique name, you are able to install and
use multiple versions of the SDK on your system.

## SDK Contents

This section describes how the SDK contents are organized.

### Applications

The `Applications` directory contains several Vulkan demos that are packaged as
macOS application "bundles".
These applications are totally self-contained and can be launched immediately
from this directory.
They can also be "installed" by "drag-and-drop" operations in `Finder` to any
desired location.

These bundles contain all libraries and resources needed to run the applications and
require no additional installation steps.

The `vulkaninfo` application opens a new `Terminal` window to display its output
since it is not a graphical windowed application.

**Note:** macOS has a security feature that prevents the opening of unsigned applications.
If you get a message saying that the application cannot be opened because it is from
an unidentified developer, you can control-click the application icon and then
select "Open" from the shortcut menu.
Then click "Open" to proceed.
See this [article](https://support.apple.com/kb/PH25088) for more information.
The exact details may vary across different releases of macOS.

### Frameworks

The `macOS/Frameworks` directory contains a Vulkan framework which contains the headers
and libraries for easy inclusion in an Xcode project.

### MoltenVK Distribution

The `MoltenVK` directory contains MoltenVK frameworks and libraries built from the
MoltenVK repository for both macOS and iOS.
You can add these to your Xcode projects to link in the MoltenVK support.
But you would tend to need these only if not using the Vulkan loader to load
the MoltenVK driver.

### "Vulkan_SDK" Tree

The `macOS` directory is the root of a "system tree" containing Vulkan executables,
include files, and libraries in the traditional `bin`, `include`, `lib` structure.
This tree contains the Vulkan components such as the loader library and layer libraries.

This tree also contains executables, include files, and libraries for various
Vulkan-related tools such as glslang, SPIRV-Tools, and others.

It is often useful to point system environment variables
(e.g., `PATH`, `DYLD_LIBRARY_PATH`)
and Vulkan-related environment variables (e.g., `VK_LAYER_PATH`) to directories
in this directory to locate the components found in this tree.

### Documentation

The `Documentation` directory contains SDK and Vulkan Documentation such as:.

* Vulkan Specification: Contains the extensions supported by MoltenVK

## SDK Origins

Much of the content found in the SDK is collected from several public GitHub
repositories and is built and packaged to form the delivered SDK.
You may wish to refer to these repositories for more information about
a particular component or if you need to obtain and build one of the
components yourself.

The primary repositories are:

* [MoltenVK](https://github.com/KhronosGroup/MoltenVK)
* [Vulkan-Headers](https://github.com/KhronosGroup/Vulkan-Headers)
* [Vulkan-Loader](https://github.com/KhronosGroup/Vulkan-Loader)
* [Vulkan-ValidationLayers](https://github.com/KhronosGroup/Vulkan-ValidationLayers)
* [Vulkan-Tools](https://github.com/KhronosGroup/Vulkan-Tools)
* [LunarG/VulkanTools](https://github.com/LunarG/VulkanTools) (planned)
* [LunarG/VulkanSamples](https://github.com/LunarG/VulkanSamples) (planned)

Several other repositories are "pulled in" by the above repositories.
Some of these are:

* [glslang](https://github.com/KhronosGroup/glslang)
* [SPIRV-Tools](https://github.com/KhronosGroup/SPIRV-Tools)
* [SPIRV-Headers](https://github.com/KhronosGroup/SPIRV-Headers)
* [SPIRV-Cross](https://github.com/KhronosGroup/SPIRV-Cross)
* [shaderc](https://github.com/google/shaderc)

If you have a problem with a component and can determine its origin from the
list above, please open a problem report in the appropriate GitHub repository.

## Options for Using Vulkan Support Components

The Vulkan SDK is flexible in the sense that it can support various workflows.
Some of these workflows are summarized here and are explained in greater detail
later in this document.

### Create a Bundled Application

Your Xcode project copies all required components from the SDK and places
them in the application bundle.
This results in a standalone application that can be copied or published anywhere
and does not require the user to have an SDK installed.

Using Xcode, add the various components from the SDK to your Xcode project.
Configure your project so all required components are in your bundle.

Note that while this results in a totally self-contained application bundle,
these components are placed into the bundle rather statically and cannot be
easily upgraded.

This presents a trade-off between potential improved stability of the application
versus the flexibility of being able to upgrade the Vulkan components.

### Xcode With SDK Side-Install

Your Xcode project has references to the SDK and requires that an SDK
be installed on your system.
This workflow might be a development phase that eventually leads
to the bundled application workflow.
This typically requires you to set Xcode Environment variables and or
Xcode custom paths to point to your SDK location.

### Install to System Directories

macOS discourages third-parties from installing libraries or frameworks to
system directories.
But it is possible for application developers to put together
their own macOS "Packages"
that come with an application and are installed when the application is installed.
This may be a good approach for putting the Vulkan components into a place where they
can be updated without updating the application.

macOS also allows users to install files to `/usr/local` which may be convenient
for users who prefer this approach.
There are no "install" scripts provided with this SDK, but you can manually
install desired SDK files to the appropriate system locations.

## Using the SDK

### Pre-built Applications

The applications in the Applications directory are simple pre-built demos that you can
launch immediately from `Finder`.

You can also start them from the command line with `open`.

### Useful Environment Variables

You may need to set environment variables inside the Xcode interface
when working with Xcode projects that use SDK components.
You may also need to set environment variables in your command shell
when running some non-bundled vulkan applications or other SDK tools
from the command line.

For these purposes, the following variables are useful.
(Replace `vulkansdk` with the path to your SDK installation.)

`PATH` - Add `vulkansdk/macOS/bin` to this path to make it easy to run
`vulkaninfo` and the various glslang and SPIRV-Tools installed there.

`DYLD_LIBRARY_PATH` - Add `vulkansdk/macOS/lib` to this path so that
executables can load the libraries installed there.
This should rarely be needed and is not necessary for
libraries that are located by JSON files, such as the ICD and layer libraries.

`VK_LAYER_PATH` - set to point at the layer JSON files in
`vulkansdk/macOS/etc/vulkan/explicit_layer.d` so that the
vulkan loader can locate the layers installed in the SDK via these JSON files.

`VK_ICD_FILENAMES` - set to point at the JSON file for the MoltenVK ICD,
which is located at `vulkansdk/macOS/etc/vulkan/icd.d/MoltenVK_icd.json`.
This ICD library itself is installed in the SDK in the SDK's `macOS/lib` directory
and is pointed to by this JSON file.

The rest of this guide explains when to use these variables.

### Command Line

The SDK contains a command line version of `vulkaninfo` in the `macOS/bin` directory.
This is a non-bundled Vulkan command line application and so needs
some environment variables set in order to function properly.

Here's how to set this up, using the `Terminal` application:

Set a convenience variable to point to the SDK:

    export VULKAN_SDK=vulkansdk/macOS

Replace the "vulkansdk" above with the actual path to your SDK.
Make sure you include the `/macOS` part.

Add the bin dir to your `PATH` so that you can run programs from there:

    export PATH=$VULKAN_SDK/bin:$PATH

Add the lib dir to your `DYLD_LIBRARY_PATH` so that programs find the Vulkan loader library:

    export DYLD_LIBRARY_PATH=$VULKAN_SDK/lib:$DYLD_LIBRARY_PATH

Now you can run `vulkaninfo` from any directory and you should see an error saying
that it cannot create a Vulkan instance.
This is because we have not told the Vulkan loader where to find a driver.
Fix this with:

    export VK_ICD_FILENAMES=$VULKAN_SDK/etc/vulkan/icd.d/MoltenVK_icd.json

Run `vulkaninfo` again and you should see something like:

    bash-3.2$ vulkaninfo
    ==========
    VULKANINFO
    ==========

    Vulkan API Version: 1.0.68

    Instance Extensions:
    ====================
    Instance Extensions count = 3
        VK_KHR_surface                      : extension revision 25
        VK_MVK_macos_surface                : extension revision  2
        VK_EXT_debug_report                 : extension revision  9

    Layers: count = 6
    =======
    VK_LAYER_LUNARG_core_validation (LunarG Validation Layer) Vulkan version 1.0.65, layer version 1
        Layer Extensions    count = 1
            VK_EXT_debug_report                 : extension revision  6
        Devices     count = 2
            GPU id       : 0 (AMD Radeon Pro 560)
            Layer-Device Extensions count = 1
                VK_EXT_debug_marker                 : extension revision  4
            GPU id       : 1 (Intel(R) HD Graphics 630)
            Layer-Device Extensions count = 1
                VK_EXT_debug_marker                 : extension revision  4

    (Output truncated for brevity)

You can also run with the validation layers enabled:

    export VK_LAYER_PATH=$VULKAN_SDK/etc/vulkan/explicit_layer.d
    VK_INSTANCE_LAYERS=VK_LAYER_LUNARG_standard_validation vulkaninfo

Since there shouldn't be any validation errors in `vulkaninfo`, you can expect
the same output.

Note that there's a bundled version of `vulkaninfo` in the SDK's Applications
directory which may be easier to use since the above configuring is not
required.
But if you use non-bundled Vulkan applications from the command line
frequently, you may wish to set these environment variables in your
login scripts or install the components to system locations.

#### Installing Vulkan Components to System Directories

macOS is comparable to other UNIX-like operating systems in that you can install
components to system directories as a "super-user".
macOS is also a bit more locked down in the sense that a super-user can't easily
install to places like `/usr/bin`.
However, it is fairly easy to install to places like `/usr/local/bin`.

This SDK currently does not directly support or encourage installing SDK components to
system directories.

But if you are interested in installing SDK components to system directories,
you can do so manually and pick only the components you need to install.
This may be helpful for reducing the need to set
environment variables to locate the components that reside in your SDK directory.

For example, to install the ICD to the system directories:

1. copy `vulkansdk/macOS/lib/libMoltenvVK.dylib` to `/usr/local/lib`
1. create directory: `/etc/vulkan/icd.d`
1. copy `vulkansdk/macOS/etc/icd.d/MoltenVK_icd.json` to `/etc/vulkan/icd.d`
1. edit `/etc/vulkan/icd.d/MoltenVK_icd.json`
    1. Change the `library_path` to remove any leading path, leaving just `libMoltenVK.dylib`

## Application Bundle Structure

The Application bundle for the `cube.app` looks like:

    cube.app
        Contents
            Frameworks
                libMoltenVK.dylib
            Info.plist
            MacOS
                cube
                libvulkan.1.0.69.dylib
                libvulkan.1.dylib -> libvulkan.1.0.69.dylib
            Resources
                LunarGIcon.icns
                Main.storyboardc
                    Info.plist
                    MainMenu.nib
                    NSWindowController-B8D-0N-5wS.nib
                    XfG-lQ-9wD-view-m2S-Jp-Qdl.nib
                vulkan
                    icd.d
                        MoltenVK_icd.json

Note that all required components are in the bundle.

The SDK also provides the Vulkan loader as a Framework in addition to a
standalone library.
This allows you to add the Vulkan loader support to your application as
a Framework if you wish.
This process is demonstrated later in this document.

### Loader Search Paths for ICD and Layers

The Vulkan loader for macOS searches the same paths for ICDs and Layers as the
Vulkan Loader for Linux.
These paths are described in
[The Loader And Layer Interface document](https://github.com/KhronosGroup/Vulkan-Loader/blob/master/loader/LoaderAndLayerInterface.md).

The macOS loader also looks in the application's bundle for ICD and layer JSON files.
It looks in the bundle's `Resources/vulkan/icd.d` directory for ICD JSON files and
in the bundle's `Resources/vulkan/explicit_layer.d` directory for layer JSON files.
The ICD JSON file (`MoltenVK_icd.json`) can be seen in the bundle shown above.

If the executable is in a bundle, the loader searches the bundle first before searching
the system directories.

## Working with Xcode

Since the Vulkan header files and loader are provided in a framework
(`macOS/Frameworks/vulkan.framework`),
it is fairly easy to add these to your Xcode project, just like any other
framework.
However, if you wish your project to have run-time access to the MoltenVK
ICD and Vulkan validation layers, you will have to guide the Xcode project to
those components residing in the SDK.

This section covers these details.

### Configuring Xcode with Environment Variables

If you want your Xcode project to refer to components residing in an
SDK instead of bundling these components, you can set environment
variables in your Xcode project to point to these components instead
of bundling them.

This may be a technique suitable for developers whose workflows are
compatible with "pointing" to a specific SDK during their work.
It is unlikely that anyone would want to deploy an application that
requires an SDK to be present.

Here are some approaches for setting these variables:

#### Set Variables in Xcode

Xcode has an interface for setting environment variables in the "Scheme"
panel.
This is a fairly common and straightforward approach.

#### Inherit Variables From Environment

Environment variables set in your shell will not be
seen by Xcode if Xcode is launched from Launchpad or other desktop GUI.
You may be able to pass shell environment settings into Xcode by starting Xcode
with `open` from a bash shell command line where the environment variables are set.
With Xcode versions 7 and later, you may need to enable this behavior:

    defaults write com.apple.dt.Xcode UseSanitizedBuildSystemEnvironment -bool NO

#### System-wide Preferences

Configure the system-wide Preferences for Xcode by adding Custom Paths in the
Preference's Locations tab.

### Frameworks and Header/Libraries

You can add Vulkan loader support to your application by adding the Vulkan framework
to your project, or by adding the Vulkan header files and Vulkan loader library file
individually to your project.
The framework approach is described in the following Xcode examples.

The other approach is similar.
You add the SDK's `macOS/include` directory to the project's Header Search Paths.
And you add both the `libvulkan.1.dylib` and the `libvulkan.1.0.xx.dylib` to your
application's folder in the Project Navigator.
Like in the Framework examples below, you may have to add a Copy Files step to
copy these two files into the compiled project.

`libvulkan.1.dylib` is a symbolic link to `libvulkan.1.0.xx.dylib`.
You may prefer to link only to `libvulkan.1.0.xx.dylib` or rename
`libvulkan.1.0.xx.dylib` to `libvulkan.1.dylib` and use `libvulkan.1.dylib`
in your project.

### Xcode Examples

Here are some simple step-by-step instructions for creating Xcode projects from
scratch that create Vulkan instances and load validation layers.

#### Create an Instance

* Create a new Project
  * Use the Command Line Tool Application template
  * Name it whatever you like
  * Choose C++ for the Language
* Modify the main program
  * Display the file `main.cpp` using the Project Navigator
  * Replace the existing code with:

```C
#include <iostream>
#include <vulkan/vulkan.h>

int main(int argc, const char * argv[]) {
    VkInstance instance;
    VkResult result;
    VkInstanceCreateInfo info = {};

    result = vkCreateInstance(&info, NULL, &instance);
    std::cout << "vkCreateInstance result: " << result  << "\n";

    vkDestroyInstance(instance, nullptr);
    return 0;
}

```

Note that Xcode will show you some errors, including not finding vulkan.h and
unknown Vulkan types and symbols.

* Add the Vulkan framework
  * Copy `macOS/Frameworks/vulkan.framework` to your application's top directory
  * Select the project in the Project Navigator and open the "Build Phases" tab
  * Open up the "Link Binary With Libraries" section
    * Drag the Vulkan Framework from your application's top directory into the list area
  * Open up the "Copy Files" section.
    * Set the Destination to Frameworks
    * Clear the Subpath
    * Uncheck "Copy only when installing"
    * Uncheck "Code Sign On Copy" if you're not set up for Code Signing
    * Drag the Vulkan Framework from your application's top directory into the list area (yes, you are adding it twice)

Return to your source code and you should see that the errors have cleared out.

If not, follow whatever procedures are necessary for adding a private framework in your
version of Xcode.
These procedures may vary across versions of Xcode.
For example, if Xcode still cannot find `vulkan.h`, add the path to the SDK include
directory to your projects header search path.

Go ahead and compile the code and run it.
You should see the `vkCreateInstance` call return a -9.
Find the vulkan_core.h file under the Frameworks area in the Project Navigator and
see that a -9 means `INCOMPATIBLE DRIVER`.
This is happening because the Vulkan loader doesn't know where to find the
MoltenVK ICD.

* Add the path to the ICD JSON file to the environment
  * Open up the Scheme panel for your project
  * Add the environment variable `VK_ICD_FILENAMES`
  * Set it to `vulkansdk/macOS/etc/vulkan/icd.d/MoltenVK_icd.json`
    * Again, "vulkansdk" is the location of the SDK you installed

Rerun the application.
The `vkCreateInstance` call should return 0.

#### Get a List of Layers

Next, we'll add code to get a list of layers installed on the system.

Update your code to look like:

```C
#include <iostream>
#include <vulkan/vulkan.h>

int main(int argc, const char * argv[]) {
    VkInstance instance;
    VkResult result;
    VkInstanceCreateInfo info = {};
    uint32_t instance_layer_count;

    result = vkEnumerateInstanceLayerProperties(&instance_layer_count, nullptr);
    std::cout << instance_layer_count << " layers found!\n";
    if (instance_layer_count > 0) {
        std::unique_ptr<VkLayerProperties[]> instance_layers(new VkLayerProperties[instance_layer_count]);
        result = vkEnumerateInstanceLayerProperties(&instance_layer_count, instance_layers.get());
        for (int i = 0; i < instance_layer_count; ++i) {
            std::cout << instance_layers[i].layerName << "\n";
        }
    }

    result = vkCreateInstance(&info, NULL, &instance);
    std::cout << "vkCreateInstance result: " << result  << "\n";

    vkDestroyInstance(instance, nullptr);
    return 0;
}

```

Compile and run this code and notice that no layers are found.
We need to tell the Vulkan loader where to find the layers.

* Add the path to the layer JSON files to the environment
  * Open up the Scheme panel for your project
  * Add the environment variable `VK_LAYER_PATH`
  * Set it to `vulkansdk/macOS/etc/vulkan/explicit_layer.d`
    * Again, "vulkansdk" is the location of the SDK you installed

Run the progam again and it should list the layers on the system.

#### Load the Validation Layers

Finally, we'll actually load the validation layers and make sure that they
are working.

Update your code:

```C
#include <iostream>
#include <vulkan/vulkan.h>

int main(int argc, const char * argv[]) {
    VkInstance instance;
    VkResult result;
    VkInstanceCreateInfo info = {};
    uint32_t instance_layer_count;

    result = vkEnumerateInstanceLayerProperties(&instance_layer_count, nullptr);
    std::cout << instance_layer_count << " layers found!\n";
    if (instance_layer_count > 0) {
        std::unique_ptr<VkLayerProperties[]> instance_layers(new VkLayerProperties[instance_layer_count]);
        result = vkEnumerateInstanceLayerProperties(&instance_layer_count, instance_layers.get());
        for (int i = 0; i < instance_layer_count; ++i) {
            std::cout << instance_layers[i].layerName << "\n";
        }
    }

    const char * names[] = {
        "VK_LAYER_LUNARG_standard_validation"
    };
    info.enabledLayerCount = 1;
    info.ppEnabledLayerNames = names;

    result = vkCreateInstance(&info, NULL, &instance);
    std::cout << "vkCreateInstance result: " << result  << "\n";

    vkDestroyInstance(instance, nullptr);
    return 0;
}
```

Compile and run and you should see a validation error message complaining that
the `sType` field is not set to the expected value.

This is not surprising because the `VkInstanceCreateInfo` structure was
simply initialized to 0, except for the validation layer name list.
You can experiment further by fixing this problem and see what happens next.

## Working with CMake

You can also build applications using this SDK with CMake.

CMake versions 3.7 and later include a `FindVulkan.cmake` module that
is useful for locating the SDK and using it in CMake projects.
This module relies on the `VULKAN_SDK` environment variable to locate
the SDK.
If this variable is not set, the module looks in the standard system
locations such as `/usr/local`.

Assuming that you have set the `VULKAN_SDK` environment variable to
the `macOS` directory in your SDK installation:

    export VULKAN_SDK=path-to-sdk/macOS

then the CMake `find_package(vulkan)` command can find the needed
components in the SDK.

### CMake Example

This example uses CMake to create both "Unix Makefiles" and Xcode projects
to build the console version of the `vulkaninfo` application.

Create an empty directory and copy the source code for `vulkaninfo` from
the Khronos Group Vulkan-Tools GitHub
[repository](https://github.com/KhronosGroup/Vulkan-Tools).
The file is `vulkaninfo/vulkaninfo.c`.

Create the file `CMakeLists.txt` and populate it with the following contents:

```CMake
cmake_minimum_required(VERSION 3.7)
project(vulkaninfo)
find_package(vulkan REQUIRED)
add_executable(vulkaninfo vulkaninfo.c)
target_link_libraries(vulkaninfo Vulkan::Vulkan)
```

Assuming that the current directory is this new directory now containing
`vulkaninfo.c` and `CMakeLists.txt`, build the project:

```script
mkdir build
cd build
cmake ..
make
```

You can then run the application:

```script
export VK_ICD_FILENAMES=$VULKAN_SDK/etc/vulkan/icd.d/MoltenVK_icd.json
./vulkaninfo
```

You can then debug the project with `lldb` or other debugger.
You may want to rebuild using:

    cmake -DCMAKE_BUILD_TYPE=Debug ..

to build with debugging information.

If you prefer, you can build an Xcode project:

```script
mkdir build
cd build
cmake -GXcode ..
open vulkaninfo.xcodeproj
```

and then work with the code in this Xcode project.
