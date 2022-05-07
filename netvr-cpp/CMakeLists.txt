cmake_minimum_required(VERSION 3.17 FATAL_ERROR)
project(isbl_netvr LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 17)

set(SRC src/main.cpp src/calibration.cpp src/calibration-eigen.cpp)

if (EMSCRIPTEN)
    add_executable(${PROJECT_NAME} ${SRC})
else()
    add_library(${PROJECT_NAME} SHARED ${SRC})
endif()

if(WIN32)
    target_compile_definitions(${PROJECT_NAME} PRIVATE WIN_EXPORT)
endif()

# Copy files to correct locations in Unity/dashboard
if (EMSCRIPTEN)
    target_link_options(${PROJECT_NAME} PRIVATE "SHELL:-s MODULARIZE=1")
    target_link_options(${PROJECT_NAME} PRIVATE "SHELL:-s ALLOW_MEMORY_GROWTH=1")
    target_link_options(${PROJECT_NAME} PRIVATE "SHELL:-s EXPORT_ES6=1")
    target_link_options(${PROJECT_NAME} PRIVATE "SHELL:-s ALLOW_TABLE_GROWTH=1")
    target_link_options(${PROJECT_NAME} PRIVATE "SHELL:-s ASSERTIONS=0")
    target_link_options(${PROJECT_NAME} PRIVATE "SHELL:-s ENVIRONMENT=web")
    target_link_options(${PROJECT_NAME} PRIVATE "SHELL:-s FILESYSTEM=0")
    target_link_options(${PROJECT_NAME} PRIVATE "SHELL:-s USE_SDL=0")
    #target_link_options(${PROJECT_NAME} PRIVATE "SHELL:-s MINIMAL_RUNTIME=1")
    target_link_options(${PROJECT_NAME} PRIVATE "SHELL:-s INCOMING_MODULE_JS_API=[]")
    #target_link_options(${PROJECT_NAME} PRIVATE "SHELL:-s USE_CLOSURE_COMPILER=1")
    target_link_options(${PROJECT_NAME} PRIVATE "SHELL:-s EXPORTED_RUNTIME_METHODS=cwrap,addFunction,UTF8ToString,getValue")
    #target_link_options(${PROJECT_NAME} PRIVATE "SHELL:-s SAFE_HEAP=1")
    target_link_options(${PROJECT_NAME} PRIVATE "SHELL:--closure 1")
    #target_compile_options(${PROJECT_NAME} PRIVATE "-g")
    #target_link_options(${PROJECT_NAME} PRIVATE "-gseparate-dwarf=${PROJECT_NAME}.debug.wasm")
    add_custom_command(TARGET ${PROJECT_NAME} POST_BUILD
        COMMAND ${CMAKE_COMMAND} -E copy $<TARGET_FILE_DIR:${PROJECT_NAME}>/${PROJECT_NAME}.js
        ${CMAKE_CURRENT_SOURCE_DIR}/../netvr-dashboard/src/wasm/${PROJECT_NAME}.js
    )
    add_custom_command(TARGET ${PROJECT_NAME} POST_BUILD
        COMMAND ${CMAKE_COMMAND} -E copy $<TARGET_FILE_DIR:${PROJECT_NAME}>/${PROJECT_NAME}.wasm
        ${CMAKE_CURRENT_SOURCE_DIR}/../netvr-dashboard/src/wasm/${PROJECT_NAME}.wasm
    )
    #add_custom_command(TARGET ${PROJECT_NAME} POST_BUILD
    #    COMMAND ${CMAKE_COMMAND} -E copy $<TARGET_FILE_DIR:${PROJECT_NAME}>/${PROJECT_NAME}.debug.wasm
    #    ${CMAKE_CURRENT_SOURCE_DIR}/../netvr-dashboard/src/wasm/${PROJECT_NAME}.debug.wasm
    #)
elseif(WIN32)
    add_custom_command(TARGET ${PROJECT_NAME} POST_BUILD
        COMMAND ${CMAKE_COMMAND} -E copy $<TARGET_FILE:${PROJECT_NAME}>
        ${CMAKE_CURRENT_SOURCE_DIR}/../netvr-unity/Assets/Plugins/Windows/${CMAKE_GENERATOR_PLATFORM}/$<TARGET_FILE_NAME:${PROJECT_NAME}>
    )
elseif(ANDROID)
    add_custom_command(TARGET ${PROJECT_NAME} POST_BUILD
        COMMAND ${CMAKE_COMMAND} -E copy $<TARGET_FILE:${PROJECT_NAME}>
        ${CMAKE_CURRENT_SOURCE_DIR}/../netvr-unity/Assets/Plugins/Android/${CMAKE_ANDROID_ARCH_ABI}/$<TARGET_FILE_NAME:${PROJECT_NAME}>
    )
endif()

if (MSVC)
  add_definitions(/bigobj)
endif ()

message(STATUS ${CMAKE_CURRENT_BINARY_DIR})
message(STATUS ${ANDROID_ARCH})
message(STATUS ${CMAKE_ANDROID_ARCH_ABI})

## Eigen
include(FetchContent)

FetchContent_Declare(
  eigen
  URL https://gitlab.com/libeigen/eigen/-/archive/3.4.0/eigen-3.4.0.tar.bz2)
FetchContent_MakeAvailable(eigen)
message(STATUS "eigen is available in " ${Eigen3_SOURCE_DIR})
target_link_libraries (${PROJECT_NAME} Eigen3::Eigen)
## Eigen END
