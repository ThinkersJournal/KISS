@echo off
REM KISS-Conform 6.6 on-device slice -- Windows build+run driver.
REM (ASCII only: cmd mis-parses non-ASCII bytes in REM lines under the OEM codepage.)
REM
REM Compiles fmax_ieee.cu with nvcc and runs it on the local GPU, propagating the
REM program's exit code (0 = PASS, 1 = divergence/FAIL, 2 = CUDA env error) so the
REM Rust integration test (tests/device.rs) can assert on it.
REM
REM Any arguments are passed straight through to nvcc, so the negative control is
REM   build_and_run.bat -DUSE_FMAXF_INTRINSIC
REM
REM Env knobs (all optional):
REM   KISS_CUDA_ARCH    GPU arch          (default sm_89, the RTX 4070 / Ada)
REM   KISS_VCVARS_VER   MSVC toolset pin  (default 14.29)
REM
REM Why the MSVC pin: CUDA 13.3's headers reject the VS 2026-era STL (toolset
REM 14.5x) with std::aligned_storage / result_of static_assert failures. The
REM newest toolset CUDA 13.3 accepts on this box is 14.29 (VS 2019). vcvarsall's
REM -vcvars_ver selects it without uninstalling anything.
setlocal EnableDelayedExpansion

if "%KISS_CUDA_ARCH%"=="" set KISS_CUDA_ARCH=sm_89
if "%KISS_VCVARS_VER%"=="" set KISS_VCVARS_VER=14.29

set SCRIPT_DIR=%~dp0

REM Locate vcvarsall.bat via vswhere (ships with every VS 2017+ installer).
set VSWHERE=%ProgramFiles(x86)%\Microsoft Visual Studio\Installer\vswhere.exe
if not exist "%VSWHERE%" set VSWHERE=%ProgramFiles%\Microsoft Visual Studio\Installer\vswhere.exe
for /f "usebackq tokens=*" %%i in (`"%VSWHERE%" -latest -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath`) do set VSINSTALL=%%i
if "%VSINSTALL%"=="" (
  echo ERROR: no Visual Studio C++ toolset found via vswhere
  exit /b 2
)
set VCVARS=%VSINSTALL%\VC\Auxiliary\Build\vcvarsall.bat

call "%VCVARS%" x64 -vcvars_ver=%KISS_VCVARS_VER% >nul 2>&1
if errorlevel 1 (
  echo ERROR: vcvarsall failed for toolset %KISS_VCVARS_VER%
  exit /b 2
)

cd /d "%SCRIPT_DIR%"
set OUT=%TEMP%\kiss_fmax_ieee_%RANDOM%.exe
nvcc -O2 -arch=%KISS_CUDA_ARCH% %* fmax_ieee.cu -o "%OUT%"
if errorlevel 1 (
  echo ERROR: nvcc compile failed
  exit /b 2
)
"%OUT%"
set RC=%errorlevel%
del "%OUT%" >nul 2>&1
exit /b %RC%
