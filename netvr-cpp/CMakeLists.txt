cmake_minimum_required(VERSION 3.17 FATAL_ERROR)
project(isbl_netvr LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 17)

add_library(${PROJECT_NAME} SHARED src/main.cpp)

if(WIN32)
    target_compile_definitions(${PROJECT_NAME} PRIVATE WIN_EXPORT)
    configure_file(
        ${CMAKE_CURRENT_BINARY_DIR}/Release/isbl_netvr.dll
        ${CMAKE_CURRENT_SOURCE_DIR}/../netvr-unity/Assets/Plugins/Windows/x64/isbl_netvr.dll
        COPYONLY)
elseif(ANDROID)
    add_custom_command(TARGET ${PROJECT_NAME} POST_BUILD
        COMMAND ${CMAKE_COMMAND} -E copy $<TARGET_FILE:${PROJECT_NAME}>
        ${CMAKE_CURRENT_SOURCE_DIR}/../netvr-unity/Assets/Plugins/Android/${CMAKE_ANDROID_ARCH_ABI}/libisbl_netvr.so
    )
endif()

message(STATUS ${CMAKE_CURRENT_BINARY_DIR})
message(STATUS ${ANDROID_ARCH})
message(STATUS ${CMAKE_ANDROID_ARCH_ABI})
